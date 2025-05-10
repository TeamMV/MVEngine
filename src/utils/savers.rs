use std::ops::Deref;
use std::sync::Arc;
use bytebuffer::{ByteBuffer, Endian};
use mvutils::save::{Loader, Savable, Saver};

pub fn save_arc_by_clone<T: Savable>(saver: &mut impl Saver, arc: &Arc<T>) {
    arc.save(saver)
}

pub fn load_arc_by_clone<T: Savable>(loader: &mut impl Loader) -> Result<Arc<T>, String> {
    let thingy = T::load(loader)?;
    Ok(Arc::new(thingy))
}

pub fn save_to_vec<T: Savable>(t: &T) -> Vec<u8> {
    let mut buffer = ByteBuffer::new();
    buffer.set_endian(Endian::LittleEndian);
    t.save(&mut buffer);
    buffer.into_vec()
}

/// An arc that implements Savable by creating a new value everytime it is loaded
/// That means that this will run:
/// ```
/// use std::ops::Deref;
/// use bytebuffer::ByteBuffer;
/// use mvutils::save::Savable;
/// use mvengine::utils::savers::SaveArc;
///
///
///
/// let arc = SaveArc::new(5i32);
///
/// //in the context of saving and loading:
/// let mut buffer = ByteBuffer::new();
/// arc.save(&mut buffer);
///
/// let arc2 = SaveArc::<i32>::load(&mut buffer).expect("Couldnt read from buffer");
///
/// let f1 = format!("{:p}", arc.deref());
/// let f2 = format!("{:p}", arc2.deref());
/// assert_ne!(f1, f2);
/// ```
pub struct SaveArc<T> {
    inner: Arc<T>
}

impl<T> SaveArc<T> {
    pub fn new(t: T) -> Self {
        Self {
            inner: Arc::new(t),
        }
    }

    pub fn arc(&self) -> &Arc<T> {
        &self.inner
    }
}

impl<T> Deref for SaveArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Savable> Savable for SaveArc<T> {
    fn save(&self, saver: &mut impl Saver) {
        save_arc_by_clone(saver, &self.inner);
    }

    fn load(loader: &mut impl Loader) -> Result<Self, String> {
        let arc = load_arc_by_clone::<T>(loader)?;
        Ok(Self { inner: arc })
    }
}

impl<T> Clone for SaveArc<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> From<Arc<T>> for SaveArc<T> {
    fn from(value: Arc<T>) -> Self {
        Self {
            inner: value
        }
    }
}