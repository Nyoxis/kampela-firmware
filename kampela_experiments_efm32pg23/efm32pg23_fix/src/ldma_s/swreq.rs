#[doc = "Register `SWREQ` writer"]
pub type W = crate::W<SwreqSpec>;
#[doc = "Field `SWREQ` writer - Software Transfer Requests"]
pub type SwreqW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl W {
    #[doc = "Bits 0:7 - Software Transfer Requests"]
    #[inline(always)]
    #[must_use]
    pub fn swreq(&mut self) -> SwreqW<SwreqSpec> {
        SwreqW::new(self, 0)
    }
}
#[doc = "No Description\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swreq::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SwreqSpec;
impl crate::RegisterSpec for SwreqSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`swreq::W`](W) writer structure"]
impl crate::Writable for SwreqSpec {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
#[doc = "`reset()` method sets SWREQ to value 0"]
impl crate::Resettable for SwreqSpec {
    const RESET_VALUE: u32 = 0;
}
