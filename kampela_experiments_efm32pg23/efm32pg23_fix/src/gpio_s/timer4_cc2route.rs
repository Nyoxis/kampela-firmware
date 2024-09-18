#[doc = "Register `TIMER4_CC2ROUTE` reader"]
pub type R = crate::R<Timer4Cc2routeSpec>;
#[doc = "Register `TIMER4_CC2ROUTE` writer"]
pub type W = crate::W<Timer4Cc2routeSpec>;
#[doc = "Field `PORT` reader - CC2 port select register"]
pub type PortR = crate::FieldReader;
#[doc = "Field `PORT` writer - CC2 port select register"]
pub type PortW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `PIN` reader - CC2 pin select register"]
pub type PinR = crate::FieldReader;
#[doc = "Field `PIN` writer - CC2 pin select register"]
pub type PinW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:1 - CC2 port select register"]
    #[inline(always)]
    pub fn port(&self) -> PortR {
        PortR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 16:19 - CC2 pin select register"]
    #[inline(always)]
    pub fn pin(&self) -> PinR {
        PinR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - CC2 port select register"]
    #[inline(always)]
    #[must_use]
    pub fn port(&mut self) -> PortW<Timer4Cc2routeSpec> {
        PortW::new(self, 0)
    }
    #[doc = "Bits 16:19 - CC2 pin select register"]
    #[inline(always)]
    #[must_use]
    pub fn pin(&mut self) -> PinW<Timer4Cc2routeSpec> {
        PinW::new(self, 16)
    }
}
#[doc = "CC2 port/pin select\n\nYou can [`read`](crate::Reg::read) this register and get [`timer4_cc2route::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`timer4_cc2route::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Timer4Cc2routeSpec;
impl crate::RegisterSpec for Timer4Cc2routeSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`timer4_cc2route::R`](R) reader structure"]
impl crate::Readable for Timer4Cc2routeSpec {}
#[doc = "`write(|w| ..)` method takes [`timer4_cc2route::W`](W) writer structure"]
impl crate::Writable for Timer4Cc2routeSpec {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
#[doc = "`reset()` method sets TIMER4_CC2ROUTE to value 0"]
impl crate::Resettable for Timer4Cc2routeSpec {
    const RESET_VALUE: u32 = 0;
}
