import { Callback } from 'ffi'
import { _ } from 'lodash'
import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { StateType } from './common'
import { Connection } from './connection'
import { VCXBaseWithState } from './VCXBaseWithState'

export interface ITrustPingConfig {
  sourceId: string,
}

/**
 */
export interface ITrustPingData {
  source_id: string
}

/**
 */
export class TrustPing extends VCXBaseWithState {
  protected _releaseFn = rustAPI().vcx_trust_ping_release
  protected _updateStFn = rustAPI().vcx_trust_ping_update_state
  protected _getStFn = rustAPI().vcx_trust_ping_get_state
  protected _serializeFn = rustAPI().vcx_trust_ping_serialize
  protected _deserializeFn = rustAPI().vcx_trust_ping_deserialize

  /**
   */
  static async create (data: ITrustPingConfig): Promise<TrustPing> {
    const obj = new TrustPing(data.sourceId)

    try {
      await obj._create((cb) => rustAPI().vcx_trust_ping_create(
        0,
        obj.sourceId,
        cb
      ))
      return obj
    } catch (err) {
      throw new VCXInternalError(`vcx_trust_ping_create -> ${err}`)
    }
  }

  /**
   */
  static async deserialize (data: ITrustPingData) {
    try {
      const obj = await super._deserialize(TrustPing, data)
      return obj
    } catch (err) {
      throw new VCXInternalError(`vcx_trust_ping_deserialize -> ${err}`)
    }
  }

  /**
   */
  async serialize (): Promise<ITrustPingData> {
    try {
      const data: ITrustPingData = JSON.parse(await super._serialize())
      return data
    } catch (err) {
      throw new VCXInternalError(`vcx_trust_ping_serialize -> ${err}`)
    }
  }

  /**
   */
  async getState (): Promise<StateType> {
    try {
      return await this._getState()
    } catch (error) {
      throw new VCXInternalError(`vcx_trust_ping_get_state -> ${error}`)
    }
  }

  /**
   */
  async updateState (): Promise<void> {
    try {
      await this._updateState()
    } catch (error) {
      throw new VCXInternalError(`vcx_trust_ping_update_state -> ${error}`)
    }
  }

  /**
   */
  async sendPing (connection: Connection): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_trust_ping_send_request(0, this.handle, connection.handle, cb)
            if (rc) {
              reject(rc)
            }
          },
          (resolve, reject) => Callback('void', ['uint32', 'uint32'], (xcommandHandle, err) => {
            if (err) {
              reject(err)
            } else {
              resolve(xcommandHandle)
            }
          })
        )
    } catch (err) {
      throw new VCXInternalError(`vcx_trust_ping_send_request -> ${err}`)
    }
  }
}
