import { Callback } from 'ffi'

import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { Connection } from './connection'
import { VCXBaseWithState } from './VCXBaseWithState'

export interface ITrusteeData {
  source_id: string,
}

export class Trustee extends VCXBaseWithState {
  protected _releaseFn = rustAPI().vcx_trustee_release
  protected _updateStFn = rustAPI().vcx_trustee_update_state
  protected _getStFn = rustAPI().vcx_trustee_get_state
  protected _serializeFn = rustAPI().vcx_trustee_serialize
  protected _deserializeFn = rustAPI().vcx_trustee_deserialize

  constructor (sourceId) {
    super(sourceId)
  }

  static async create (sourceId: string, offer: string): Promise<Trustee> {
    const trustee = new Trustee(sourceId)
    try {
      await trustee._create((cb) => rustAPI().vcx_trustee_create_with_offer(
        0,
        sourceId,
        offer,
        cb
        )
      )
      return trustee
    } catch (err) {
      throw new VCXInternalError(`vcx_trustee_create_with_offer -> ${err}`)
    }
  }

  static async deserialize (data: ITrusteeData) {
    try {
      const attr = {}
      const trustee = await super._deserialize<Trustee, {}>(Trustee, data, attr)
      return trustee
    } catch (err) {
      throw new VCXInternalError(`vcx_trustee_deserialize -> ${err}`)
    }
  }

  static async new_offers (connection: Connection): Promise<string> {
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_trustee_new_offers(0, connection.handle, cb)
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
      throw new VCXInternalError(`vcx_trustee_get_state -> ${error}`)
    }
  }

  async updateState (): Promise<void> {
    try {
      await this._updateState()
    } catch (error) {
      throw new VCXInternalError(`vcx_trustee_update_state -> ${error}`)
    }
  }

  async serialize (): Promise<ITrusteeData> {
    try {
      return JSON.parse(await super._serialize())
    } catch (err) {
      throw new VCXInternalError(`vcx_trustee_serialize -> ${err}`)
    }
  }

  async sendRequest (connection: Connection): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_trustee_send_request(0, this.handle, connection.handle, cb)
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
      throw new VCXInternalError(`vcx_trustee_send_request -> ${err}`)
    }
  }
}
