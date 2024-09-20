use aya_ebpf_bindings::bindings::xdp_action;

pub struct BoundsError;

#[macro_export]
macro_rules! bounds {
    ($ctx:expr, $size:expr) => {
        if ($ctx.data() + $size > $ctx.data_end()) {
            Err($crate::aya::BoundsError)
        } else {
            Ok(())
        }
    };
}

#[inline(always)]
pub fn csum_fold_helper(mut csum: u64) -> u16 {
    for _i in 0..4 {
        if (csum >> 16) > 0 {
            csum = (csum & 0xffff) + (csum >> 16);
        }
    }
    !(csum as u16)
}

#[inline(always)]
pub fn csum_diff<T: Copy>(mut old: T, mut new: T, seed: u32) -> u64 {
    unsafe {
        aya_ebpf_bindings::helpers::bpf_csum_diff(
            (&mut old) as *mut T as *mut _,
            size_of::<T>() as u32,
            (&mut new) as *mut T as *mut _,
            size_of::<T>() as u32,
            seed,
        ) as u64
    }
}

pub trait XdpErr<T> {
    fn or_drop(self) -> Result<T, u32>
    where
        T: core::marker::Sized;
    fn or_pass(self) -> Result<T, u32>
    where
        T: core::marker::Sized;
    fn or_abort(self) -> Result<T, u32>
    where
        T: core::marker::Sized;
}

impl<T, E> XdpErr<T> for Result<T, E> {
    fn or_drop(self) -> Result<T, u32>
    where
        T: core::marker::Sized,
    {
        self.map_err(|_| xdp_action::XDP_DROP)
    }

    fn or_pass(self) -> Result<T, u32>
    where
        T: core::marker::Sized,
    {
        self.map_err(|_| xdp_action::XDP_PASS)
    }

    fn or_abort(self) -> Result<T, u32>
    where
        T: core::marker::Sized,
    {
        self.map_err(|_| xdp_action::XDP_ABORTED)
    }
}
