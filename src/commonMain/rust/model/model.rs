use prost::Message;

use stremio_core::models::addon_details::AddonDetails;
use stremio_core::models::catalog_with_filters::CatalogWithFilters;
use stremio_core::models::catalogs_with_extra::CatalogsWithExtra;
use stremio_core::models::continue_watching_preview::ContinueWatchingPreview;
use stremio_core::models::ctx::Ctx;
use stremio_core::models::library_by_type::LibraryByType;
use stremio_core::models::library_with_filters::{LibraryWithFilters, NotRemovedFilter};
use stremio_core::models::link::Link;
use stremio_core::models::meta_details::MetaDetails;
use stremio_core::models::player::Player;
use stremio_core::models::streaming_server::StreamingServer;
use stremio_core::runtime::Effects;
use stremio_core::types::api::LinkAuthKey;
use stremio_core::types::events::DismissedEventsBucket;
use stremio_core::types::library::LibraryBucket;
use stremio_core::types::notifications::NotificationsBucket;
use stremio_core::types::profile::Profile;
use stremio_core::types::resource::MetaItemPreview;
use stremio_core::types::search_history::SearchHistoryBucket;
use stremio_core::types::streams::StreamsBucket;
use stremio_core::Model;

use stremio_core_protobuf::bridge::ToProtobuf;

use crate::env::AppleEnv;
use stremio_core_protobuf::model::AddonsWithFilters;

#[derive(Model, Clone)]
#[model(AppleEnv)]
pub struct AppleModel {
    pub ctx: Ctx,
    pub auth_link: Link<LinkAuthKey>,
    pub continue_watching_preview: ContinueWatchingPreview,
    pub discover: CatalogWithFilters<MetaItemPreview>,
    pub library: LibraryWithFilters<NotRemovedFilter>,
    pub library_by_type: LibraryByType<NotRemovedFilter>,
    pub board: CatalogsWithExtra,
    pub search: CatalogsWithExtra,
    pub meta_details: MetaDetails,
    pub addons: AddonsWithFilters,
    pub addon_details: AddonDetails,
    pub streaming_server: StreamingServer,
    pub player: Player,
}

impl AppleModel {
    pub fn new(
        profile: Profile,
        library: LibraryBucket,
        streams: StreamsBucket,
        notifications: NotificationsBucket,
        search_history: SearchHistoryBucket,
        dismissed_events: DismissedEventsBucket,
    ) -> (AppleModel, Effects) {
        let (continue_watching_preview, continue_watching_preview_effects) =
            ContinueWatchingPreview::new(&library, &notifications);

        let ctx = Ctx::new(
            profile,
            library,
            streams,
            notifications,
            search_history,
            dismissed_events,
        );

        let (discover, discover_effects) = CatalogWithFilters::<MetaItemPreview>::new(&ctx.profile);
        let (library_, library_effects) =
            LibraryWithFilters::<NotRemovedFilter>::new(&ctx.library, &ctx.notifications);
        let (library_by_type, library_by_type_effects) = LibraryByType::<NotRemovedFilter>::new();
        let (addons, addons_effects) = AddonsWithFilters::new(&ctx.profile);
        let (streaming_server, streaming_server_effects) =
            StreamingServer::new::<AppleEnv>(&ctx.profile);
        let model = AppleModel {
            ctx,
            auth_link: Default::default(),
            continue_watching_preview,
            discover,
            library: library_,
            library_by_type,
            board: Default::default(),
            search: Default::default(),
            meta_details: Default::default(),
            addons,
            addon_details: Default::default(),
            streaming_server,
            player: Default::default(),
        };
        (
            model,
            continue_watching_preview_effects
                .join(discover_effects)
                .join(library_effects)
                .join(library_by_type_effects)
                .join(addons_effects)
                .join(streaming_server_effects),
        )
    }

    pub fn get_state_binary(&self, field: &AppleModelField) -> Vec<u8> {
        match field {
            AppleModelField::Ctx => self.ctx.to_protobuf::<AppleEnv>(&()).encode_to_vec(),
            AppleModelField::AuthLink => self.auth_link.to_protobuf::<AppleEnv>(&()).encode_to_vec(),
            AppleModelField::ContinueWatchingPreview => self
                .continue_watching_preview
                .to_protobuf::<AppleEnv>(&self.ctx)
                .encode_to_vec(),
            AppleModelField::Library => self.library.to_protobuf::<AppleEnv>(&self.ctx).encode_to_vec(),
            AppleModelField::LibraryByType => {
                self.library_by_type.to_protobuf::<AppleEnv>(&self.ctx).encode_to_vec()
            }
            AppleModelField::Board => self.board.to_protobuf::<AppleEnv>(&self.ctx).encode_to_vec(),
            AppleModelField::Search => self.search.to_protobuf::<AppleEnv>(&self.ctx).encode_to_vec(),
            AppleModelField::Discover => self.discover.to_protobuf::<AppleEnv>(&self.ctx).encode_to_vec(),
            AppleModelField::MetaDetails => {
                self.meta_details.to_protobuf::<AppleEnv>(&self.ctx).encode_to_vec()
            }
            AppleModelField::Addons => self.addons.to_protobuf::<AppleEnv>(&self.ctx).encode_to_vec(),
            AppleModelField::AddonDetails => {
                self.addon_details.to_protobuf::<AppleEnv>(&self.ctx).encode_to_vec()
            }
            AppleModelField::StreamingServer => {
                self.streaming_server.to_protobuf::<AppleEnv>(&()).encode_to_vec()
            }
            AppleModelField::Player => self.player.to_protobuf::<AppleEnv>(&self.ctx).encode_to_vec(),
        }
    }
}
