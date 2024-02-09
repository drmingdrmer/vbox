use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use vbox::from_vbox;
use vbox::into_vbox;
use vbox::VBox;

#[test]
fn test_fn_once() {
    struct NoSend {
        p: PhantomData<*const ()>,
    }

    let n = NoSend {
        p: Default::default(),
    };

    let cnt = Arc::new(AtomicU64::new(0));

    let f = {
        let a = cnt.clone();
        move |_st: &_| {
            a.fetch_add(1, Ordering::Relaxed);
        }
    };

    assert_eq!(0, cnt.load(Ordering::Relaxed));

    let vb: VBox = into_vbox!(dyn FnOnce(&NoSend), f);
    let f2: Box<dyn FnOnce(&NoSend)> = from_vbox!(dyn FnOnce(&NoSend), vb);

    f2(&n);

    assert_eq!(1, cnt.load(Ordering::Relaxed));
}

#[test]
fn test_debug() {
    let v = 3u64;

    let vb: VBox = into_vbox!(dyn Debug, v);
    let p: Box<dyn Debug> = from_vbox!(dyn Debug, vb);

    let got = format!("{:?}", p);
    assert_eq!("3", got);
}

#[test]
fn test_plus() {
    trait Plus {
        fn plus(&self, s: u64) -> u64;
    }

    impl Plus for u64 {
        fn plus(&self, s: u64) -> u64 {
            self + s
        }
    }

    let v = 3u64;

    let vb: VBox = into_vbox!(dyn Plus, v);
    let p: Box<dyn Plus> = from_vbox!(dyn Plus, vb);

    let got = p.plus(1);
    assert_eq!(4, got);
}

#[test]
fn test_drop() {
    trait Plus {
        fn plus(&self, s: u64) -> u64;
    }

    struct Foo {
        a: Arc<AtomicU64>,
    }

    impl Plus for Foo {
        fn plus(&self, s: u64) -> u64 {
            s
        }
    }

    let drop_cnt = Arc::new(AtomicU64::new(0));

    impl Drop for Foo {
        fn drop(&mut self) {
            self.a.fetch_add(1, Ordering::Relaxed);
        }
    }

    let v = Foo {
        a: drop_cnt.clone(),
    };
    assert_eq!(0, drop_cnt.load(Ordering::Relaxed));

    {
        let _vb: VBox = into_vbox!(dyn Plus, v);
    }

    assert_eq!(1, drop_cnt.load(Ordering::Relaxed), "drop is called");

    let v = Foo {
        a: drop_cnt.clone(),
    };
    {
        let vb: VBox = into_vbox!(dyn Plus, v);
        let _p: Box<dyn Plus> = from_vbox!(dyn Plus, vb);
    }
    assert_eq!(2, drop_cnt.load(Ordering::Relaxed), "drop is called");
}
