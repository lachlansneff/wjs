use ouroboros::self_referencing;

use crate::gc::{Gc, GcPtr};

#[self_referencing]
struct VMInner {
    gc: Box<Gc>,
    #[borrows(gc)]
    #[covariant]
    roots: Roots<'this>,
}

pub struct VM {
    inner: VMInner,
}

struct Roots<'gc> {
    test: GcPtr<'gc, f64>,
}

impl VM {
    pub fn new() -> Self {
        let inner = VMInnerBuilder {
            gc: Box::new(Gc::new()),
            roots_builder: |gc| {
                Roots {
                    test: gc.alloc(5.1),
                }
            },
        }.build();

        Self {
            inner,
        }
    }

    // pub fn 
}
