use crate::bridge::{FromProtobuf, ToProtobuf};
use crate::protobuf::stremio::core::models;
use stremio_core::models::addon_details::{AddonDetails, Selected};
use stremio_core::models::ctx::Ctx;

impl FromProtobuf<Selected> for models::addon_details::Selected {
    fn from_protobuf(&self) -> Selected {
        let mut url = self.transport_url.from_protobuf();
        if url.scheme() == "stremio" {
            let replaced_url = url.as_str().replacen("stremio://", "https://", 1);
            url = replaced_url.parse().expect("Should be able to parse URL");
        }
        Selected { transport_url: url }
    }
}

impl ToProtobuf<models::addon_details::Selected, ()> for Selected {
    fn to_protobuf(&self, _args: &()) -> models::addon_details::Selected {
        models::addon_details::Selected {
            transport_url: self.transport_url.to_string(),
        }
    }
}

impl ToProtobuf<models::AddonDetails, Ctx> for AddonDetails {
    fn to_protobuf(&self, ctx: &Ctx) -> models::AddonDetails {
        models::AddonDetails {
            selected: self.selected.to_protobuf(&()),
            local_addon: self.local_addon.to_protobuf(ctx),
            remote_addon: self.remote_addon.to_protobuf(ctx),
        }
    }
}
