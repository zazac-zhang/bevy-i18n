mod asset;
mod component;
pub mod interpolate;
mod plugin;
mod resource;
mod systems;

/// Prelude — one-import convenience
pub mod prelude {
    pub use crate::asset::I18nAsset;
    pub use crate::component::T;
    pub use crate::interpolate::NumberFormat;
    pub use crate::plugin::I18nPlugin;
    pub use crate::resource::I18n;
}

pub use crate::interpolate::NumberFormat;
