import { Callback } from 'ffi'

import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { StateType } from './common'
import { Connection } from './connection'
import { VCXBaseWithState } from './VCXBaseWithState'

export interface IClaimOfferVCXAttributes {
  [ index: string ]: [ string ]
}

/**
 * @interface
 * @description
 * SourceId: String for SDK User's reference.
 * issuerDid: DID associated with the claim def.
 * attributes: key: [value] list of items offered in claim
 */
export interface IClaimOfferConfig {
  sourceId: string,
  schemaNum: number,
  attr: {
    IClaimOfferVCXAttributes
  },
  claimName: string,
}

export interface IClaimOfferParams {
  schemaNum: number,
  claimName: string,
  attr: IClaimOfferVCXAttributes
}

interface IClaimOfferMessage {
  msg_type: string,
  version: string,
  to_did: string,
  from_did: string,
  claim: {
    [ index: string ]: [ string ]
  },
  schema_seq_no: number,
  issuer_did: string,
  claim_name: string,
  claim_id: string,
  msg_ref_id: any
}

export interface IClaimOfferData {
  source_id: string,
  handle: number,
  schema_seq_no: number,
  claim_attributes: string,
  claim_name: string,
  issuer_did: string,
  state: StateType
}

export type IClaimOffer = string

export interface IClaimCreateData {
  sourceId: string,
  offer: IClaimOffer
}

export type IClaimOffer = string

export interface IClaimCreateData {
  sourceId: string,
  offer: IClaimOffer
}

export class Claim extends VCXBaseWithState {
  protected _releaseFn = rustAPI().vcx_claim_release
  protected _updateStFn = rustAPI().vcx_claim_update_state
  protected _getStFn = rustAPI().vcx_claim_get_state
  protected _serializeFn = rustAPI().vcx_claim_serialize
  protected _deserializeFn = rustAPI().vcx_claim_deserialize
  private _schemaNum: number
  private _issuerDID: string
  private _claimName: string
  private _attr: IClaimOfferVCXAttributes

  constructor (sourceId, { schemaNum, claimName, attr }: IClaimOfferParams) {
    super(sourceId)
    this._schemaNum = schemaNum
    this._claimName = claimName
    this._attr = attr
  }

  /**
   * @memberof Claim
   * @description Builds a generic Claim object
   * @static
   * @async
   * @function create_with_message
   * @param sourceId
   * @param message
   * @example <caption>Example of message</caption>
   * {
   *   "msg_type":"CLAIM_OFFER",
   *   "version":"0.1",
   *   "to_did":"LtMgSjtFcyPwenK9SHCyb8",
   *   "from_did":"LtMgSjtFcyPwenK9SHCyb8",
   *   "claim":{
   *     "account_num":[
   *       "8BEaoLf8TBmK4BUyX8WWnA"
   *     ],
   *     "name_on_account":[
   *       "Alice"
   *     ]
   *   },
   *   "schema_seq_no":48,
   *   "issuer_did":"Pd4fnFtRBcMKRVC2go5w3j",
   *   "claim_name":"Account Certificate",
   *   "claim_id":"3675417066",
   *   "msg_ref_id":null
   * }
   * @example <caption>Example of IClaimOfferConfig</caption>
   * { sourceId: "48", attr: {key: "value"}, claimName: "Account Certificate"}
   * @returns {Promise<Claim>} A Claim Object
   */
  static async create_with_message (sourceId: string, message: string): Promise<Claim> {
    /* TODO: ensure the parsed JSON message contains all required fields */
    const offerJSON: IClaimOfferMessage = JSON.parse(message)
    const attrsVCX: IClaimOfferVCXAttributes = Object.keys(offerJSON.claim || {})
      .reduce((accum, attrKey) => ({ ...accum, [attrKey]: [offerJSON.claim[attrKey][0]] }), {})
    const claim = new Claim(sourceId, {
      attr: attrsVCX,
      claimName: offerJSON.claim_name,
      schemaNum: offerJSON.schema_seq_no
    })
    const offer = message
    try {
      await claim._create((cb) => rustAPI().vcx_claim_create_with_offer(
        0,
        sourceId,
        offer,
        cb
        )
      )
      return claim
    } catch (err) {
      throw new VCXInternalError(`vcx_claim_create_with_offer -> ${err}`)
    }
  }

  static async deserialize (claimData: IClaimOfferData) {
    try {
      const attr = JSON.parse(claimData.claim_attributes)
      const params: IClaimOfferParams = {
        attr,
        claimName: claimData.claim_name,
        schemaNum: claimData.schema_seq_no
      }
      const claim = await super._deserialize<Claim, IClaimOfferParams>(Claim, claimData, params)
      return claim
    } catch (err) {
      throw new VCXInternalError(`vcx_issuer_claim_deserialize -> ${err}`)
    }
  }

  static async new_offers (connection: Connection): Promise<IClaimOffer[]> {
    const offersStr = await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_claim_new_offers(0, connection.handle, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback('void', ['uint32', 'uint32', 'string'], (handle, err, messages) => {
        if (err) {
          reject(err)
        } else {
          resolve(messages)
        }
      })
    )
    const offers = JSON.parse(offersStr)
    return offers
  }

  async getState (): Promise<StateType> {
    try {
      return await this._getState()
    } catch (error) {
      throw new VCXInternalError(`vcx_claim_get_state -> ${error}`)
    }
  }

  async updateState (): Promise<void> {
    try {
      await this._updateState()
    } catch (error) {
      throw new VCXInternalError(`vcx_claim_update_state -> ${error}`)
    }
  }

  async serialize (): Promise<IClaimOfferData> {
    try {
      return JSON.parse(await super._serialize())
    } catch (err) {
      throw new VCXInternalError(`vcx_claim_serialize -> ${err}`)
    }
  }

  async sendRequest (connection: Connection): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_claim_send_request(0, this.handle, connection.handle, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => Callback('void', ['uint32', 'uint32'], (xcommandHandle, err) => {
            if (err) {
              reject(err)
            } else {
              resolve()
            }
          })
        )
    } catch (err) {
      // TODO handle error
      throw new VCXInternalError(`vcx_claim_send_request -> ${err}`)
    }
  }

  get claimName () {
    return this._claimName
  }

  get issuerDid () {
    return this._issuerDID
  }

  get attr () {
    return this._attr
  }
}
