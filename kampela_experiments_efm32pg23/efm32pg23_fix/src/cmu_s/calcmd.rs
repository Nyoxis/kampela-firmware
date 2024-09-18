#[doc = "Register `CALCMD` writer"]
pub type W = crate::W<CalcmdSpec>;
#[doc = "Field `CALSTART` writer - Calibration Start"]
pub type CalstartW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CALSTOP` writer - Calibration Stop"]
pub type CalstopW<'a, REG> = crate::BitWriter<'a, REG>;
impl W {
    #[doc = "Bit 0 - Calibration Start"]
    #[inline(always)]
    #[must_use]
    pub fn calstart(&mut self) -> CalstartW<CalcmdSpec> {
        CalstartW::new(self, 0)
    }
    #[doc = "Bit 1 - Calibration Stop"]
    #[inline(always)]
    #[must_use]
    pub fn calstop(&mut self) -> CalstopW<CalcmdSpec> {
        CalstopW::new(self, 1)
    }
}
#[doc = "No Description\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`calcmd::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CalcmdSpec;
impl crate::RegisterSpec for CalcmdSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`calcmd::W`](W) writer structure"]
impl crate::Writable for CalcmdSpec {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
#[doc = "`reset()` method sets CALCMD to value 0"]
impl crate::Resettable for CalcmdSpec {
    const RESET_VALUE: u32 = 0;
}
