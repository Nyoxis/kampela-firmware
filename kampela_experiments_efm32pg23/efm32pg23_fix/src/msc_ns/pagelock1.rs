#[doc = "Register `PAGELOCK1` reader"]
pub type R = crate::R<Pagelock1Spec>;
#[doc = "Register `PAGELOCK1` writer"]
pub type W = crate::W<Pagelock1Spec>;
#[doc = "Field `LOCKBIT` reader - page lock bit"]
pub type LockbitR = crate::FieldReader<u32>;
#[doc = "Field `LOCKBIT` writer - page lock bit"]
pub type LockbitW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - page lock bit"]
    #[inline(always)]
    pub fn lockbit(&self) -> LockbitR {
        LockbitR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - page lock bit"]
    #[inline(always)]
    #[must_use]
    pub fn lockbit(&mut self) -> LockbitW<Pagelock1Spec> {
        LockbitW::new(self, 0)
    }
}
#[doc = "No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`pagelock1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pagelock1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Pagelock1Spec;
impl crate::RegisterSpec for Pagelock1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pagelock1::R`](R) reader structure"]
impl crate::Readable for Pagelock1Spec {}
#[doc = "`write(|w| ..)` method takes [`pagelock1::W`](W) writer structure"]
impl crate::Writable for Pagelock1Spec {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
#[doc = "`reset()` method sets PAGELOCK1 to value 0"]
impl crate::Resettable for Pagelock1Spec {
    const RESET_VALUE: u32 = 0;
}
