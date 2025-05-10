use mvengine_proc_macro::r;

pub mod err;
pub mod runtime;

pub const CR: usize = usize::MAX / 2; //Custom Resources start

use crate as mvengine;
use crate::math::vec::Vec4;
use crate::rendering::texture::Texture;
use crate::ui::context::UiResources;

r! {
    <resources structName="MVR" cdir="./" superSecretTagWhichSpecifiesThisIsTheMVResourceStruct="andItsSuperSecretValue">
        <colors>
            <color name="white" val="white"/>
            <color name="black" val="black"/>
            <color name="red" val="red"/>
            <color name="green" val="green"/>
            <color name="blue" val="blue"/>
            <color name="yellow" val="yellow"/>
            <color name="magenta" val="magenta"/>
            <color name="cyan" val="cyan"/>
            <color name="transparent" val="transparent"/>
        </colors>
        <shapes>
            <shape name="rect" src="shapes/rect.msf"/>
            <shape name="round_rect" src="shapes/round_rect.msf"/>
        </shapes>
        <adaptives>
            <adaptive name="void_rect" src="shapes/void_rect.msf"/>
        </adaptives>
        <textures>
            <texture name="test" src="textures/img.png"/>
            <texture name="missing" src="textures/missing.png"/>
        </textures>
        <fonts>
            <font name="default" src="fonts/data.font" atlas="fonts/atlas.png"/>
        </fonts>
        <tilesets>
            <tileset name="smiley" atlas="textures/test_tileset.png" width="128" height="128" count="16">
                <entry name="happy" index="4"/>
                <fps value="24"/>
            </tileset>
        </tilesets>
        <animations>
            <animation name="smiley" tileset="smiley" range="..8" fps="12"/>
        </animations>
        <composites>
            <composite name="test" rig="rigs/test.mrf">
                <part res="animation.smiley"/>
                <part res="tile.smiley.happy"/>
                <part res="texture.test"/>
            </composite>
        </composites>
    </resources>
}

pub trait OrMissingTexture<T> {
    fn or_missing_texture(self) -> T;
}

impl OrMissingTexture<&Texture> for Option<&'static Texture> {
    fn or_missing_texture(self) -> &'static Texture {
        if let Some(val) = self {
            val
        } else {
            MVR.resolve_texture(MVR.texture.missing)
                .expect("Missing texture must exist")
        }
    }
}

impl OrMissingTexture<(&Texture, Vec4)> for Option<(&'static Texture, Vec4)> {
    fn or_missing_texture(self) -> (&'static Texture, Vec4) {
        if let Some(val) = self {
            val
        } else {
            (
                MVR.resolve_texture(MVR.texture.missing)
                    .expect("Missing texture must exist"),
                Vec4::default_uv(),
            )
        }
    }
}
