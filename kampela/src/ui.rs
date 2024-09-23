//! Everything high-level related to interfacing with user
use alloc::{borrow::ToOwned, string::String, vec::Vec};
use substrate_crypto_light::sr25519::{Pair, Public};
use embedded_graphics::geometry::Dimensions;

use kampela_system::{
    devices::{
        psram::{psram_decode_call, psram_decode_extension, read_from_psram, PsramAccess},
        flash::{store_encoded_entopy, read_encoded_entropy},
        se_aes_gcm::{decode_entropy, encode_entropy, ProtectedPair},
        se_rng,
    }, draw::FrameBuffer, flash_mnemonic::FlashWordList, parallel::Operation
};
use crate::{nfc::NfcTransactionPsramAccess, touch::get_touch_point};
use kampela_ui::{
    platform::{PinCode, Platform},
    uistate::{UIState, UpdateRequest, UpdateRequestMutate}
};

/// UI handler
pub struct UI {
    pub state: UIState<Hardware, FrameBuffer>,
    update_request: Option<UpdateRequest>,
    ui_status: UIStatus,
}

impl UI {
    /// Start of UI.
    pub fn init() -> Self {
        let hardware = Hardware::new();
        let display = FrameBuffer::new_white();
        let state = UIState::new(hardware, display, &mut ());
        return Self {
            state,
            update_request: Some(UpdateRequest::Slow),
            ui_status: UIStatus::Listen,
        }
    }

    /// Call in event loop to progress through UI state
    pub fn advance(&mut self, voltage: i32) -> Option<bool> {
        match self.ui_status {
            UIStatus::Listen => {
                let a = self.listen();
                if a.unwrap_or(false) {
                    cortex_m::asm::wfi(); // sleep waiting for tocuh irq
                }
                a
            },
            UIStatus::DisplayOperation => {
                match self.state.display.advance(voltage) {
                    Some(c) => {
                        if c {
                            self.ui_status = UIStatus::Listen;
                            Some(true)
                        } else {
                            Some(false)
                        }
                    },
                    None => None, // not enough energy to start screen update
                }
            },
        }
    }

    fn listen(&mut self) -> Option<bool> {
        if let Some(point) = get_touch_point() {
            self.update_request.propagate(self.state.handle_tap(point, &mut ()));
        }
        // update ui if needed
        if let Some(u) = self.update_request.take() {
            let is_clear_update = matches!(u, UpdateRequest::Slow) || matches!(u, UpdateRequest::Fast);
            self.update_request.propagate(self.state.render(is_clear_update, &mut ()).expect("guaranteed to work, no errors implemented"));

            match u {
                UpdateRequest::Hidden => (),
                UpdateRequest::Slow => self.state.display.request_full(),
                UpdateRequest::Fast => self.state.display.request_fast(),
                UpdateRequest::UltraFast => {
                    let a = self.state.display.bounding_box();
                    self.state.display.request_part(a);
                },
                UpdateRequest::Part(a) => self.state.display.request_part(a),
            }
            if !matches!(u, UpdateRequest::Hidden) {
                self.ui_status = UIStatus::DisplayOperation;
            }
            None
        } else {
            Some(true) // done operations
        }
    }

    pub fn handle_message(&mut self, message: String) {
        self.update_request.propagate(self.state.handle_message(message, &mut ()));
    }

    pub fn handle_transaction(&mut self, transaction: NfcTransactionPsramAccess) {
        self.state.platform.set_transaction(transaction);
        self.update_request.propagate(self.state.handle_transaction(&mut ()));
    }

    pub fn handle_address(&mut self, addr: [u8; 76]) {
        self.update_request.propagate(self.state.handle_address(addr));
    }
}

/// General status of UI
///
/// There is no sense in reading input while screen processes last event, nor refreshing the screen
/// before touch was parsed
enum UIStatus {
    /// Event listening state, default
    Listen,
    /// Screen update started
    DisplayOperation,
}
impl Default for UIStatus {
    fn default() -> Self { UIStatus::Listen }
}

pub struct Hardware {
    pin: PinCode,
    protected_pair: Option<ProtectedPair>,
    address: Option<[u8; 76]>,
    transaction_psram_access: Option<NfcTransactionPsramAccess>,
}

impl Hardware {
    pub fn new() -> Self {
        let protected_pair = None;
        let pin_set = false; // TODO query storage
        let pin = [0; 4];
        Self {
            pin,
            protected_pair,
            address: None,
            transaction_psram_access: None,
        }
    }
}

impl Platform for Hardware {
    type HAL = ();
    type Rng<'c> = se_rng::SeRng;
    type AsWordList = FlashWordList;

    type NfcTransaction = NfcTransactionPsramAccess;
    fn get_wordlist() -> Self::AsWordList {
        FlashWordList::new()
    }

    fn rng<'b>(_: &'b mut ()) -> Self::Rng<'static> {
        se_rng::SeRng{}
    }

    fn pin(&self) -> &PinCode {
        &self.pin
    }

    fn pin_mut(&mut self) -> &mut PinCode {
        &mut self.pin
    }

    fn store_entropy(&mut self, e: &[u8]) {
        self.protected_pair = if e.len() != 0 {
            let protected = encode_entropy(e);
            let public = Pair::from_entropy_and_pwd(&e, "").unwrap().public();
            let protected_pair = ProtectedPair{protected, public};
            store_encoded_entopy(&protected_pair);
            Some(protected_pair)
        } else {
            None
        }
    }

    fn read_entropy(&mut self) {
        self.protected_pair = read_encoded_entropy();
    }

    fn public(&self) -> Option<Public> {
        self.protected_pair.as_ref().map(|p| p.public).to_owned()
    }

    fn entropy(&self) -> Option<Vec<u8>> {
        if let Some(p) = &self.protected_pair {
            Some(decode_entropy(&p.protected))
        } else {
            None
        }
    }

    fn set_address(&mut self, addr: [u8; 76]) {
        self.address = Some(addr);
    }

    fn set_transaction(&mut self, transaction: Self::NfcTransaction) {
        self.transaction_psram_access = Some(transaction);
    }


    fn call(&mut self) -> Option<String> {
        let transaction_psram_access = match self.transaction_psram_access {
            Some(ref a) => a,
            None => return None
        };

        let (decoded_call, specs, spec_name) = psram_decode_call(
            &transaction_psram_access.call_psram_access,
            &transaction_psram_access.metadata_psram_access,
        );

        let carded = decoded_call.card(0, &specs, &spec_name);
        let call = carded
            .into_iter()
            .map(|card| card.show())
            .collect::<Vec<String>>()
            .join("\n");

        Some(call)
    }

    fn extensions(&mut self) -> Option<String> {
        let transaction_psram_access = match self.transaction_psram_access {
            Some(ref a) => a,
            None => return None
        };
        
        let (decoded_extension, specs, spec_name) = psram_decode_extension(
            &transaction_psram_access.extension_psram_access,
            &transaction_psram_access.metadata_psram_access,
            &transaction_psram_access.genesis_hash_bytes_psram_access
        );

        let mut carded = Vec::new();
        for ext in decoded_extension.iter() {
            let addition_set = ext.card(0, true, &specs, &spec_name);
            if !addition_set.is_empty() {
                carded.extend_from_slice(&addition_set)
            }
        }
        let extensions = carded
            .into_iter()
            .map(|card| card.show())
            .collect::<Vec<String>>()
            .join("\n");

        Some(extensions)
    }

    fn signature(&mut self) -> [u8; 130] {
        let transaction_psram_access = match self.transaction_psram_access {
            Some(ref a) => a,
            None => panic!("qr generation failed")
        };
        
        let data_to_sign_psram_access = PsramAccess {
            start_address: transaction_psram_access.call_psram_access.start_address,
            total_len:
                transaction_psram_access.call_psram_access.total_len
                + &transaction_psram_access.extension_psram_access.total_len
        };
        let data_to_sign = read_from_psram(&data_to_sign_psram_access);

        let signature = self.pair()
            .expect("entropy should be stored at this point")
            .sign_external_rng(&data_to_sign, &mut Self::rng(&mut ()));

        let mut signature_with_id: [u8; 65] = [1; 65];
        signature_with_id[1..].copy_from_slice(&signature.0);
        let signature_with_id_bytes = hex::encode(signature_with_id)
            .into_bytes()
            .try_into()
            .expect("static length");

        signature_with_id_bytes
    }

    fn address(&mut self) -> &[u8; 76] {
        if let Some(ref a) = self.address {
            a
        } else {
            panic!("qr generation failed");
        }
    }

}

