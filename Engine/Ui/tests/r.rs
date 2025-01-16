use uiproc::r;
use std::any::TypeId;

r! {
    <resources structName="R" cdir="./res/">
        <colors>
            <color name="red" val="red"/>
        </colors>
        <shapes>
            <shape name="rect" src="shapes/test.msf"/>
        </shapes>
    </resources>
}