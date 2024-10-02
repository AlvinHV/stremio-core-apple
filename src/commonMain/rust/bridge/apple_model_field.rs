use stremio_core::runtime::Env;

use stremio_core_protobuf::{
    bridge::{FromProtobuf, ToProtobuf},
    protobuf::stremio::core::runtime::Field,
};

use crate::{env::AppleEnv, model::AppleModelField};

impl From<Field> for AppleModelField {
    fn from(apple_field: Field) -> Self {
        apple_field.from_protobuf()
    }
}

impl From<AppleModelField> for Field {
    fn from(field: AppleModelField) -> Self {
        field.to_protobuf::<AppleEnv>(&())
    }
}

impl FromProtobuf<AppleModelField> for Field {
    fn from_protobuf(&self) -> AppleModelField {
        match self {
            Field::Ctx => AppleModelField::Ctx,
            Field::AuthLink => AppleModelField::AuthLink,
            // Field::DataExport => AppleModelField::DataExport,
            Field::DataExport => unimplemented!(),
            Field::ContinueWatchingPreview => AppleModelField::ContinueWatchingPreview,
            Field::Board => AppleModelField::Board,
            Field::Discover => AppleModelField::Discover,
            Field::Library => AppleModelField::Library,
            Field::LibraryByType => AppleModelField::LibraryByType,
            Field::ContinueWatching => unimplemented!(),
            Field::Search => AppleModelField::Search,
            Field::LocalSearch => unimplemented!(),
            Field::Addons => AppleModelField::Addons,
            Field::MetaDetails => AppleModelField::MetaDetails,
            Field::AddonDetails => AppleModelField::AddonDetails,
            Field::StreamingServer => AppleModelField::StreamingServer,
            Field::Player => AppleModelField::Player,
        }
    }
}

impl ToProtobuf<Field, ()> for AppleModelField {
    fn to_protobuf<E: Env + 'static>(&self, _args: &()) -> Field {
        match self {
            AppleModelField::Ctx => Field::Ctx,
            AppleModelField::AuthLink => Field::AuthLink,
            AppleModelField::ContinueWatchingPreview => Field::ContinueWatchingPreview,
            AppleModelField::Discover => Field::Discover,
            AppleModelField::Library => Field::Library,
            AppleModelField::LibraryByType => Field::LibraryByType,
            AppleModelField::Board => Field::Board,
            AppleModelField::Search => Field::Search,
            AppleModelField::MetaDetails => Field::MetaDetails,
            AppleModelField::Addons => Field::Addons,
            AppleModelField::AddonDetails => Field::AddonDetails,
            AppleModelField::StreamingServer => Field::StreamingServer,
            AppleModelField::Player => Field::Player,
        }
    }
}
