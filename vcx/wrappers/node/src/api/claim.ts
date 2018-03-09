import { Callback } from 'ffi'

import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { StateType } from './common'
import { Connection } from './connection'
import { VCXBaseWithState } from './VCXBaseWithState'

export interface IClaimData {
  source_id: string,
}

export class Claim extends VCXBaseWithState {
  protected _releaseFn = rustAPI().vcx_claim_release
  protected _updateStFn = rustAPI().vcx_claim_update_state
  protected _getStFn = rustAPI().vcx_claim_get_state
  protected _serializeFn = rustAPI().vcx_claim_serialize
  protected _deserializeFn = rustAPI().vcx_claim_deserialize

  constructor (sourceId, { }) {
    super(sourceId)
  }

  static async create (sourceId: string, offer: string): Promise<Claim> {
    const claim = new Claim(sourceId, offer)
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

  static async deserialize (claimData: IClaimData) {
    try {
      const attr = {}
      const claim = await super._deserialize<Claim, {}>(Claim, claimData, attr)
      return claim
    } catch (err) {
      throw new VCXInternalError(`vcx_issuer_claim_deserialize -> ${err}`)
    }
  }

  static async new_offers (connection: Connection): Promise<string> {
    return await createFFICallbackPromise<string>(
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
  }

  async getState (): Promise<number> {
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

  async serialize (): Promise<IClaimData> {
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
              return
            }
            resolve(xcommandHandle)
          })
        )
    } catch (err) {
      // TODO handle error
      throw new VCXInternalError(`vcx_claim_send_request -> ${err}`)
    }
  }
}
