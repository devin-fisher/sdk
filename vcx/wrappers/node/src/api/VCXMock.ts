// 1 -> create keys response
// 2 -> update profile response
// 3 -> get_message response for connection acceptance
// 4 -> get_message response for claim offer
// 5 -> get_message response for updating proof state
// 6 -> get_message response with claim req
// 7 -> get_message response with proof
// 8 => UPDATE_TRUSTEE_OFFER_RESPONSE,
// 9 => TRUSTEE_REQUEST_RESPONSE,
// 10 => NEW_OFFER_RESPONSE,
// 11 => TRUSTEE_DATA_RESPONSE,
import { rustAPI } from '../rustlib'

export class VCXMock {
  static setVcxMock (messageIndex: number) {
    rustAPI().vcx_set_next_agency_response(messageIndex)
  }
}
