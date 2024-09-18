#[doc = "Register `PRS0_ASYNCH11ROUTE` reader"]
pub type R = crate::R<Prs0Asynch11routeSpec>;
#[doc = "Register `PRS0_ASYNCH11ROUTE` writer"]
pub type W = crate::W<Prs0Asynch11routeSpec>;
#[doc = "Field `PORT` reader - ASYNCH11 port select register"]
pub type PortR = crate::FieldReader;
#[doc = "Field `PORT` writer - ASYNCH11 port select register"]
pub type PortW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `PIN` reader - ASYNCH11 pin select register"]
pub type PinR = crate::FieldReader;
#[doc = "Field `PIN` writer - ASYNCH11 pin select register"]
pub type PinW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:1 - ASYNCH11 port select register"]
    #[inline(always)]
    pub fn port(&self) -> PortR {
        PortR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 16:19 - ASYNCH11 pin select register"]
    #[inline(always)]
    pub fn pin(&self) -> PinR {
        PinR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - ASYNCH11 port select register"]
    #[inline(always)]
    #[must_use]
    pub fn port(&mut self) -> PortW<Prs0Asynch11routeSpec> {
        PortW::new(self, 0)
    }
    #[doc = "Bits 16:19 - ASYNCH11 pin select register"]
    #[inline(always)]
    #[must_use]
    pub fn pin(&mut self) -> PinW<Prs0Asynch11routeSpec> {
        PinW::new(self, 16)
    }
}
#[doc = "ASYNCH11 port/pin select\n\nYou can [`read`](crate::Reg::read) this register and get [`prs0_asynch11route::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`prs0_asynch11route::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Prs0Asynch11routeSpec;
impl crate::RegisterSpec for Prs0Asynch11routeSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`prs0_asynch11route::R`](R) reader structure"]
impl crate::Readable for Prs0Asynch11routeSpec {}
#[doc = "`write(|w| ..)` method takes [`prs0_asynch11route::W`](W) writer structure"]
impl crate::Writable for Prs0Asynch11routeSpec {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
#[doc = "`reset()` method sets PRS0_ASYNCH11ROUTE to value 0"]
impl crate::Resettable for Prs0Asynch11routeSpec {
    const RESET_VALUE: u32 = 0;
}
