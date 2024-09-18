#[doc = "Register `LINKLOAD` writer"]
pub type W = crate::W<LinkloadSpec>;
#[doc = "Field `LINKLOAD` writer - DMA Link Loads"]
pub type LinkloadW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl W {
    #[doc = "Bits 0:7 - DMA Link Loads"]
    #[inline(always)]
    #[must_use]
    pub fn linkload(&mut self) -> LinkloadW<LinkloadSpec> {
        LinkloadW::new(self, 0)
    }
}
#[doc = "No Description\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`linkload::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LinkloadSpec;
impl crate::RegisterSpec for LinkloadSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`linkload::W`](W) writer structure"]
impl crate::Writable for LinkloadSpec {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
#[doc = "`reset()` method sets LINKLOAD to value 0"]
impl crate::Resettable for LinkloadSpec {
    const RESET_VALUE: u32 = 0;
}
