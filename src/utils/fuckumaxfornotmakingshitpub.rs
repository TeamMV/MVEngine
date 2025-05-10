use mvutils::once::CreateOnce;
use mvutils::save::{Loader, Savable, Saver};
use std::cell::UnsafeCell;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::panic::RefUnwindSafe;
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, Once};

pub struct MveLazy<T> {
    value: CreateOnce<T>,
    init: Mutex<Option<fn() -> T>>,
}

impl<T> MveLazy<T> {
    pub const fn new(f: fn() -> T) -> Self {
        Self {
            value: CreateOnce::new(),
            init: Mutex::new(Some(f)),
        }
    }

    pub fn created(&self) -> bool {
        self.value.created()
    }
    
    pub fn init_now(t: T) -> Self {
        let value: CreateOnce<T> = CreateOnce::new();
        value.create(|| t);
        value.deref(); //init now
        MveLazy {
            value,
            init: Mutex::new(None),
        }
    }
}

impl<T: Default> MveLazy<T> {
    pub const fn default() -> Self {
        Self {
            value: CreateOnce::new(),
            init: Mutex::new(Some(T::default)),
        }
    }
}

impl<T> Deref for MveLazy<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let mut f = self.init.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(f) = f.take() {
            self.value.create(f);
        }
        &self.value
    }
}

impl<T> DerefMut for MveLazy<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let mut f = self.init.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(f) = f.take() {
            self.value.create(f);
        }
        &mut self.value
    }
}

impl<T: Display> Display for MveLazy<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: Savable> Savable for MveLazy<T> {
    fn save(&self, saver: &mut impl Saver) {
        Deref::deref(self).save(saver);
    }

    fn load(loader: &mut impl Loader) -> Result<Self, String> {
        let t = T::load(loader)?;
        Ok(Self::init_now(t))
    }
}

unsafe impl<T: Send> Send for MveLazy<T> {}
unsafe impl<T: Sync> Sync for MveLazy<T> {}
impl<T> RefUnwindSafe for MveLazy<T> {}


#[macro_export]
macro_rules! mve_lazy {
    {
        $(
            $v:vis static $n:ident $($k:ident)?: $t:ty = $init:expr;
        )*
    } => {
        $(
            $v static $n $($k)?: $crate::utils::fuckumaxfornotmakingshitpub::MveLazy<$t> = $crate::utils::fuckumaxfornotmakingshitpub::MveLazy::new(|| { $init });
        )*
    };
    {
        $(
            let $n:ident $($k:ident)?$(: $t:ty)? = $init:expr;
        )*
    } => {
        $(
            let $n $($k)?$(: $crate::utils::fuckumaxfornotmakingshitpub::MveLazy<$t>)? = $crate::utils::fuckumaxfornotmakingshitpub::MveLazy::new(|| { $init });
        )*
    };
}

pub struct ThreadSafe<T> {
    inner: T
}

impl<T> ThreadSafe<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
    
    pub fn as_ref(&self) -> &T {
        &self.inner
    }
    
    pub fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
    
    pub fn into_inner(self) -> T {
        self.inner
    }
}

unsafe impl<T> Send for ThreadSafe<T> {}
unsafe impl<T> Sync for ThreadSafe<T> {}

pub trait CloneableFn<T>: Fn(T) + Send + Sync {
    fn clone_box(&self) -> Box<dyn CloneableFn<T>>;
}

impl<T, I> CloneableFn<I> for T
where
    T: 'static + Fn(I) + Clone + Send + Sync,
{
    fn clone_box(&self) -> Box<dyn CloneableFn<I>> {
        Box::new(self.clone())
    }
}

impl<I: 'static> Clone for Box<dyn CloneableFn<I>> {
    fn clone(&self) -> Box<dyn CloneableFn<I>> {
        self.clone_box()
    }
}

#[macro_export]
macro_rules! enum_val_ref_mut {
    (
        $en:ident, $inp:ident, $var:tt
    ) => {{
        match $inp {
            $en::$var(ref mut i) => i,
            _ => panic!("Illegal Variant"),
        }
    }};
}