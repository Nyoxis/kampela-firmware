#[doc = "Register `TXDATA` writer"]
pub type W = crate::W<TxdataSpec>;
#[doc = "Field `TXDATA` writer - TX Data"]
pub type TxdataW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl W {
    #[doc = "Bits 0:7 - TX Data"]
    #[inline(always)]
    #[must_use]
    pub fn txdata(&mut self) -> TxdataW<TxdataSpec> {
        TxdataW::new(self, 0)
    }
}
#[doc = "No Description\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`txdata::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TxdataSpec;
impl crate::RegisterSpec for TxdataSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`txdata::W`](W) writer structure"]
impl crate::Writable for TxdataSpec {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
#[doc = "`reset()` method sets TXDATA to value 0"]
impl crate::Resettable for TxdataSpec {
    const RESET_VALUE: u32 = 0;
}
