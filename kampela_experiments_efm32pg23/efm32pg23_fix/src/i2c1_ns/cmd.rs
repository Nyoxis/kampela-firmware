#[doc = "Register `CMD` writer"]
pub type W = crate::W<CmdSpec>;
#[doc = "Field `START` writer - Send start condition"]
pub type StartW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `STOP` writer - Send stop condition"]
pub type StopW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ACK` writer - Send ACK"]
pub type AckW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `NACK` writer - Send NACK"]
pub type NackW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CONT` writer - Continue transmission"]
pub type ContW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ABORT` writer - Abort transmission"]
pub type AbortW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CLEARTX` writer - Clear TX"]
pub type CleartxW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CLEARPC` writer - Clear Pending Commands"]
pub type ClearpcW<'a, REG> = crate::BitWriter<'a, REG>;
impl W {
    #[doc = "Bit 0 - Send start condition"]
    #[inline(always)]
    #[must_use]
    pub fn start(&mut self) -> StartW<CmdSpec> {
        StartW::new(self, 0)
    }
    #[doc = "Bit 1 - Send stop condition"]
    #[inline(always)]
    #[must_use]
    pub fn stop(&mut self) -> StopW<CmdSpec> {
        StopW::new(self, 1)
    }
    #[doc = "Bit 2 - Send ACK"]
    #[inline(always)]
    #[must_use]
    pub fn ack(&mut self) -> AckW<CmdSpec> {
        AckW::new(self, 2)
    }
    #[doc = "Bit 3 - Send NACK"]
    #[inline(always)]
    #[must_use]
    pub fn nack(&mut self) -> NackW<CmdSpec> {
        NackW::new(self, 3)
    }
    #[doc = "Bit 4 - Continue transmission"]
    #[inline(always)]
    #[must_use]
    pub fn cont(&mut self) -> ContW<CmdSpec> {
        ContW::new(self, 4)
    }
    #[doc = "Bit 5 - Abort transmission"]
    #[inline(always)]
    #[must_use]
    pub fn abort(&mut self) -> AbortW<CmdSpec> {
        AbortW::new(self, 5)
    }
    #[doc = "Bit 6 - Clear TX"]
    #[inline(always)]
    #[must_use]
    pub fn cleartx(&mut self) -> CleartxW<CmdSpec> {
        CleartxW::new(self, 6)
    }
    #[doc = "Bit 7 - Clear Pending Commands"]
    #[inline(always)]
    #[must_use]
    pub fn clearpc(&mut self) -> ClearpcW<CmdSpec> {
        ClearpcW::new(self, 7)
    }
}
#[doc = "No Description\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmd::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CmdSpec;
impl crate::RegisterSpec for CmdSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`cmd::W`](W) writer structure"]
impl crate::Writable for CmdSpec {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
#[doc = "`reset()` method sets CMD to value 0"]
impl crate::Resettable for CmdSpec {
    const RESET_VALUE: u32 = 0;
}
