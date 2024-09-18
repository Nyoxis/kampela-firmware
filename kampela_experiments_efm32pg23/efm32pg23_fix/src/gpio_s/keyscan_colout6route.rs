#[doc = "Register `KEYSCAN_COLOUT6ROUTE` reader"]
pub type R = crate::R<KeyscanColout6routeSpec>;
#[doc = "Register `KEYSCAN_COLOUT6ROUTE` writer"]
pub type W = crate::W<KeyscanColout6routeSpec>;
#[doc = "Field `PORT` reader - COLOUT6 port select register"]
pub type PortR = crate::FieldReader;
#[doc = "Field `PORT` writer - COLOUT6 port select register"]
pub type PortW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `PIN` reader - COLOUT6 pin select register"]
pub type PinR = crate::FieldReader;
#[doc = "Field `PIN` writer - COLOUT6 pin select register"]
pub type PinW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:1 - COLOUT6 port select register"]
    #[inline(always)]
    pub fn port(&self) -> PortR {
        PortR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 16:19 - COLOUT6 pin select register"]
    #[inline(always)]
    pub fn pin(&self) -> PinR {
        PinR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - COLOUT6 port select register"]
    #[inline(always)]
    #[must_use]
    pub fn port(&mut self) -> PortW<KeyscanColout6routeSpec> {
        PortW::new(self, 0)
    }
    #[doc = "Bits 16:19 - COLOUT6 pin select register"]
    #[inline(always)]
    #[must_use]
    pub fn pin(&mut self) -> PinW<KeyscanColout6routeSpec> {
        PinW::new(self, 16)
    }
}
#[doc = "COLOUT6 port/pin select\n\nYou can [`read`](crate::Reg::read) this register and get [`keyscan_colout6route::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`keyscan_colout6route::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct KeyscanColout6routeSpec;
impl crate::RegisterSpec for KeyscanColout6routeSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`keyscan_colout6route::R`](R) reader structure"]
impl crate::Readable for KeyscanColout6routeSpec {}
#[doc = "`write(|w| ..)` method takes [`keyscan_colout6route::W`](W) writer structure"]
impl crate::Writable for KeyscanColout6routeSpec {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
#[doc = "`reset()` method sets KEYSCAN_COLOUT6ROUTE to value 0"]
impl crate::Resettable for KeyscanColout6routeSpec {
    const RESET_VALUE: u32 = 0;
}
