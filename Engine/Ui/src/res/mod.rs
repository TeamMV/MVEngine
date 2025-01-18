use uiproc::r;

use crate as mvengine_ui;

pub mod err;

pub const CR: usize = usize::MAX / 2; //Custom Resources start

r! {
    <resources structName="MVR" cdir="../../resources/" superSecretTagWhichSpecifiesThisIsTheMVResourceStruct="andItsSuperSecretValue">
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
        </shapes>
        <adaptives>
            <adaptive name="void_rect" src="shapes/void_rect.msf"/>
        </adaptives>
        <textures>
            <texture name="test" src="textures/img.png"/>
        </textures>
    </resources>
}