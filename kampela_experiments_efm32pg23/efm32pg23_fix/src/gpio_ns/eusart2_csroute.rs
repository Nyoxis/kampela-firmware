#[doc = "Register `EUSART2_CSROUTE` reader"]
pub type R = crate::R<Eusart2CsrouteSpec>;
#[doc = "Register `EUSART2_CSROUTE` writer"]
pub type W = crate::W<Eusart2CsrouteSpec>;
#[doc = "Field `PORT` reader - CS port select register"]
pub type PortR = crate::FieldReader;
#[doc = "Field `PORT` writer - CS port select register"]
pub type PortW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `PIN` reader - CS pin select register"]
pub type PinR = crate::FieldReader;
#[doc = "Field `PIN` writer - CS pin select register"]
pub type PinW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:1 - CS port select register"]
    #[inline(always)]
    pub fn port(&self) -> PortR {
        PortR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 16:19 - CS pin select register"]
    #[inline(always)]
    pub fn pin(&self) -> PinR {
        PinR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - CS port select register"]
    #[inline(always)]
    #[must_use]
    pub fn port(&mut self) -> PortW<Eusart2CsrouteSpec> {
        PortW::new(self, 0)
    }
    #[doc = "Bits 16:19 - CS pin select register"]
    #[inline(always)]
    #[must_use]
    pub fn pin(&mut self) -> PinW<Eusart2CsrouteSpec> {
        PinW::new(self, 16)
    }
}
#[doc = "CS port/pin select\n\nYou can [`read`](crate::Reg::read) this register and get [`eusart2_csroute::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`eusart2_csroute::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Eusart2CsrouteSpec;
impl crate::RegisterSpec for Eusart2CsrouteSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`eusart2_csroute::R`](R) reader structure"]
impl crate::Readable for Eusart2CsrouteSpec {}
#[doc = "`write(|w| ..)` method takes [`eusart2_csroute::W`](W) writer structure"]
impl crate::Writable for Eusart2CsrouteSpec {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
#[doc = "`reset()` method sets EUSART2_CSROUTE to value 0"]
impl crate::Resettable for Eusart2CsrouteSpec {
    const RESET_VALUE: u32 = 0;
}
