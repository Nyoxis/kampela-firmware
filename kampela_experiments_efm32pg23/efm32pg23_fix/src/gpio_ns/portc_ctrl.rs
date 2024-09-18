#[doc = "Register `PORTC_CTRL` reader"]
pub type R = crate::R<PortcCtrlSpec>;
#[doc = "Register `PORTC_CTRL` writer"]
pub type W = crate::W<PortcCtrlSpec>;
#[doc = "Field `SLEWRATE` reader - Slew Rate"]
pub type SlewrateR = crate::FieldReader;
#[doc = "Field `SLEWRATE` writer - Slew Rate"]
pub type SlewrateW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `DINDIS` reader - Data In Disable"]
pub type DindisR = crate::BitReader;
#[doc = "Field `DINDIS` writer - Data In Disable"]
pub type DindisW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SLEWRATEALT` reader - Slew Rate Alt"]
pub type SlewratealtR = crate::FieldReader;
#[doc = "Field `SLEWRATEALT` writer - Slew Rate Alt"]
pub type SlewratealtW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `DINDISALT` reader - Data In Disable Alt"]
pub type DindisaltR = crate::BitReader;
#[doc = "Field `DINDISALT` writer - Data In Disable Alt"]
pub type DindisaltW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bits 4:6 - Slew Rate"]
    #[inline(always)]
    pub fn slewrate(&self) -> SlewrateR {
        SlewrateR::new(((self.bits >> 4) & 7) as u8)
    }
    #[doc = "Bit 12 - Data In Disable"]
    #[inline(always)]
    pub fn dindis(&self) -> DindisR {
        DindisR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bits 20:22 - Slew Rate Alt"]
    #[inline(always)]
    pub fn slewratealt(&self) -> SlewratealtR {
        SlewratealtR::new(((self.bits >> 20) & 7) as u8)
    }
    #[doc = "Bit 28 - Data In Disable Alt"]
    #[inline(always)]
    pub fn dindisalt(&self) -> DindisaltR {
        DindisaltR::new(((self.bits >> 28) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 4:6 - Slew Rate"]
    #[inline(always)]
    #[must_use]
    pub fn slewrate(&mut self) -> SlewrateW<PortcCtrlSpec> {
        SlewrateW::new(self, 4)
    }
    #[doc = "Bit 12 - Data In Disable"]
    #[inline(always)]
    #[must_use]
    pub fn dindis(&mut self) -> DindisW<PortcCtrlSpec> {
        DindisW::new(self, 12)
    }
    #[doc = "Bits 20:22 - Slew Rate Alt"]
    #[inline(always)]
    #[must_use]
    pub fn slewratealt(&mut self) -> SlewratealtW<PortcCtrlSpec> {
        SlewratealtW::new(self, 20)
    }
    #[doc = "Bit 28 - Data In Disable Alt"]
    #[inline(always)]
    #[must_use]
    pub fn dindisalt(&mut self) -> DindisaltW<PortcCtrlSpec> {
        DindisaltW::new(self, 28)
    }
}
#[doc = "Port control\n\nYou can [`read`](crate::Reg::read) this register and get [`portc_ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`portc_ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PortcCtrlSpec;
impl crate::RegisterSpec for PortcCtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`portc_ctrl::R`](R) reader structure"]
impl crate::Readable for PortcCtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`portc_ctrl::W`](W) writer structure"]
impl crate::Writable for PortcCtrlSpec {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
#[doc = "`reset()` method sets PORTC_CTRL to value 0x0040_0040"]
impl crate::Resettable for PortcCtrlSpec {
    const RESET_VALUE: u32 = 0x0040_0040;
}
