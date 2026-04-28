mod asset;
mod component;
pub mod interpolate;
mod plugin;
mod resource;
pub mod systems;

/// Prelude — one-import convenience
pub mod prelude {
    pub use crate::asset::I18nAsset;
    pub use crate::component::{I18nMarker, Localizable, TVar};
    pub use crate::interpolate::NumberFormat;
    pub use crate::plugin::I18nPlugin;
    pub use crate::resource::I18n;
    pub use crate::systems::update_localizable;

    #[cfg(feature = "derive")]
    pub use bevy_i18n_derive::I18n;
}

pub use crate::interpolate::NumberFormat;

#[cfg(feature = "derive")]
pub use crate::component::Localizable;

#[cfg(feature = "derive")]
pub use bevy_i18n_derive::I18n;
