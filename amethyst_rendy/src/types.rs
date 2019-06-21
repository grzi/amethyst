//! 'Global' rendering type declarations
use amethyst_assets::{Asset, Handle, TypeUuid};
use amethyst_core::ecs::DenseVecStorage;
use type_uuid::*;
use serde::{Deserialize, Serialize};

/// Extension of the rendy Backend trait.
pub trait Backend: rendy::hal::Backend {
    /// Unwrap a Backend to a rendy `Mesh`
    fn unwrap_mesh(mesh: &Mesh) -> Option<&rendy::mesh::Mesh<Self>>;
    /// Unwrap a Backend to a rendy `Texture`
    fn unwrap_texture(texture: &Texture) -> Option<&rendy::texture::Texture<Self>>;
    /// Wrap a rendy `Mesh` to its Backend generic.
    fn wrap_mesh(mesh: rendy::mesh::Mesh<Self>) -> Mesh;
    /// Wrap a rendy `Texture` to its Backend generic.
    fn wrap_texture(texture: rendy::texture::Texture<Self>) -> Texture;
}

macro_rules! impl_backends {
    ($($variant:ident, $feature:literal, $backend:ty;)*) => {


        impl_single_default!($([$feature, $backend]),*);

        static_assertions::assert_cfg!(
            any($(feature = $feature),*),
            concat!("You must specify at least one graphical backend feature: ", stringify!($($feature),* "See the wiki article https://github.com/amethyst/amethyst/wiki/GraphicalBackendError for more details."))
        );

        /// Backend wrapper.
        #[derive(Debug)]
        pub enum BackendVariant {
            $(
                #[cfg(feature = $feature)]
                #[doc = "Backend Variant"]
                $variant,
            )*
        }

        /// Mesh wrapper.
        #[derive(Debug, TypeUuid)]
        #[uuid = "3017f6f7-b9fa-4d55-8cc5-27f803592569"]
        pub enum Mesh {
            $(
                #[cfg(feature = $feature)]
                #[doc = "Mesh Variant"]
                $variant(rendy::mesh::Mesh<$backend>),
            )*
        }

        /// Texture wrapper.
        #[derive(Debug, TypeUuid)]
        #[uuid = "af14628f-c707-4921-9ac1-f6ae42b8ee8e"]
        pub enum Texture {
            $(
                #[cfg(feature = $feature)]
                #[doc = "Texture Variant"]
                $variant(rendy::texture::Texture<$backend>),
            )*
        }

        $(
            #[cfg(feature = $feature)]
            impl Backend for $backend {
                #[inline]
                #[allow(irrefutable_let_patterns)]
                fn unwrap_mesh(mesh: &Mesh) -> Option<&rendy::mesh::Mesh<Self>> {
                    if let Mesh::$variant(inner) = mesh {
                        Some(inner)
                    } else {
                        None
                    }
                }
                #[inline]
                #[allow(irrefutable_let_patterns)]
                fn unwrap_texture(texture: &Texture) -> Option<&rendy::texture::Texture<Self>> {
                    if let Texture::$variant(inner) = texture {
                        Some(inner)
                    } else {
                        None
                    }
                }
                #[inline]
                fn wrap_mesh(mesh: rendy::mesh::Mesh<Self>) -> Mesh {
                    Mesh::$variant(mesh)
                }
                #[inline]
                fn wrap_texture(texture: rendy::texture::Texture<Self>) -> Texture {
                    Texture::$variant(texture)
                }
            }
        )*
    };
}

// Create `DefaultBackend` type alias only when exactly one backend is selected.
macro_rules! impl_single_default {
    ( $([$feature:literal, $backend:ty]),* ) => {
        impl_single_default!(@ (), ($([$feature, $backend])*));
    };
    (@ ($($prev:literal)*), ([$cur:literal, $backend:ty]) ) => {
        #[cfg(all( feature = $cur, not(any($(feature = $prev),*)) ))]
        #[doc = "Default backend"]
        pub type DefaultBackend = $backend;
    };
    (@ ($($prev:literal)*), ([$cur:literal, $backend:ty] $([$nf:literal, $nb:ty])*) ) => {
        #[cfg(all( feature = $cur, not(any($(feature = $prev,)* $(feature = $nf),*)) ))]
        #[doc = "Default backend"]
        pub type DefaultBackend = $backend;

        impl_single_default!(@ ($($prev)* $cur), ($([$nf, $nb])*) );
    };
}

impl_backends!(
    // DirectX 12 is currently disabled because of incomplete gfx-hal support for it.
    // It will be re-enabled when it actually works.
    // Dx12, "dx12", rendy::dx12::Backend; 
    Metal, "metal", rendy::metal::Backend;
    Vulkan, "vulkan", rendy::vulkan::Backend;
    Empty, "empty", rendy::empty::Backend;
);

impl Asset for Mesh {
    fn name() -> &'static str  { "Mesh" }
    type Data = MeshData;
    type HandleStorage = DenseVecStorage<Handle<Self>>;
}

impl Asset for Texture {
    fn name() -> &'static str { "Texture" }
    type Data = TextureData;
    type HandleStorage = DenseVecStorage<Handle<Self>>;
}

/// Newtype for MeshBuilder prefab usage.
#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "c5870fe0-1733-4fb4-827c-4353f8c6002d"]
pub struct MeshData(
    #[serde(deserialize_with = "deserialize_data")] pub rendy::mesh::MeshBuilder<'static>,
);
amethyst_assets::register_asset_type!(MeshData => Mesh);

/// Newtype for TextureBuilder prefab usage.
#[derive(Debug, Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "25063afd-6cc0-487e-982f-a63fed7d7393"]
pub struct TextureData(pub rendy::texture::TextureBuilder<'static>);

amethyst_assets::register_asset_type!(TextureData => Texture);

impl From<rendy::mesh::MeshBuilder<'static>> for MeshData {
    fn from(builder: rendy::mesh::MeshBuilder<'static>) -> Self {
        Self(builder)
    }
}

impl From<rendy::texture::TextureBuilder<'static>> for TextureData {
    fn from(builder: rendy::texture::TextureBuilder<'static>) -> Self {
        Self(builder)
    }
}

fn deserialize_data<'de, D>(deserializer: D) -> Result<rendy::mesh::MeshBuilder<'static>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    Ok(rendy::mesh::MeshBuilder::deserialize(deserializer)?.into_owned())
}
