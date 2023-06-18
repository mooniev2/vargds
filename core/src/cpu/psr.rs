/// Program status register.
#[repr(transparent)]
pub struct Psr(u32);

impl Default for Psr {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl Psr {
    pub fn new() -> Self {
        Self(0)
    }

    /// Get carry.
    #[inline(always)]
    pub fn c(&self) -> bool {
        get_bit!(self.0, 29)
    }

    /// Set carry.
    #[inline(always)]
    pub fn c_set(&mut self, v: bool) {
        toggle_bit!(self.0, 29, v)
    }

    /// Get overflow.
    #[inline(always)]
    pub fn v(&mut self) -> bool {
        get_bit!(self.0, 28)
    }

    /// Set overflow.
    #[inline(always)]
    pub fn v_set(&mut self, v: bool) {
        toggle_bit!(self.0, 28, v)
    }

    /// Get zero.
    #[inline(always)]
    pub fn z(&mut self) -> bool {
        get_bit!(self.0, 30)
    }

    /// Set zero.
    #[inline(always)]
    pub fn z_set(&mut self, v: bool) {
        toggle_bit!(self.0, 30, v)
    }

    /// Get negative.
    #[inline(always)]
    pub fn n(&mut self) -> bool {
        get_bit!(self.0, 31)
    }

    /// Set negative.
    #[inline(always)]
    pub fn n_set(&mut self, v: bool) {
        toggle_bit!(self.0, 31, v)
    }

    #[inline(always)]
    pub fn raw(&self) -> u32 {
        self.0
    }
}
