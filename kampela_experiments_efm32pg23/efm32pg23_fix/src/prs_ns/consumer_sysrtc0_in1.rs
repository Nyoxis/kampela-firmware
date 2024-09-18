#[doc = "Register `CONSUMER_SYSRTC0_IN1` reader"]
pub type R = crate::R<ConsumerSysrtc0In1Spec>;
#[doc = "Register `CONSUMER_SYSRTC0_IN1` writer"]
pub type W = crate::W<ConsumerSysrtc0In1Spec>;
#[doc = "Field `PRSSEL` reader - IN1 async channel select"]
pub type PrsselR = crate::FieldReader;
#[doc = "Field `PRSSEL` writer - IN1 async channel select"]
pub type PrsselW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - IN1 async channel select"]
    #[inline(always)]
    pub fn prssel(&self) -> PrsselR {
        PrsselR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - IN1 async channel select"]
    #[inline(always)]
    #[must_use]
    pub fn prssel(&mut self) -> PrsselW<ConsumerSysrtc0In1Spec> {
        PrsselW::new(self, 0)
    }
}
#[doc = "IN1 Consumer register\n\nYou can [`read`](crate::Reg::read) this register and get [`consumer_sysrtc0_in1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`consumer_sysrtc0_in1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ConsumerSysrtc0In1Spec;
impl crate::RegisterSpec for ConsumerSysrtc0In1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`consumer_sysrtc0_in1::R`](R) reader structure"]
impl crate::Readable for ConsumerSysrtc0In1Spec {}
#[doc = "`write(|w| ..)` method takes [`consumer_sysrtc0_in1::W`](W) writer structure"]
impl crate::Writable for ConsumerSysrtc0In1Spec {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
#[doc = "`reset()` method sets CONSUMER_SYSRTC0_IN1 to value 0"]
impl crate::Resettable for ConsumerSysrtc0In1Spec {
    const RESET_VALUE: u32 = 0;
}
