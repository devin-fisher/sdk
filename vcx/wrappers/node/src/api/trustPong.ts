import { Callback } from 'ffi'

import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { StateType } from './common'
import { Connection } from './connection'
import { VCXBaseWithState } from './VCXBaseWithState'

export interface ITrustPongData {
  source_id: string,
}

export type ITrustPongRequest = string

export interface ITrustPongCreateData {
  sourceId: string,
  request: ITrustPongRequest
}

export class TrustPong extends VCXBaseWithState {
  protected _releaseFn = rustAPI().vcx_trust_pong_release
  protected _updateStFn = rustAPI().vcx_trust_pong_update_state
  protected _getStFn = rustAPI().vcx_trust_pong_get_state
  protected _serializeFn = rustAPI().vcx_trust_pong_serialize
  protected _deserializeFn = rustAPI().vcx_trust_pong_deserialize

  static async create ({ sourceId, request }: ITrustPongCreateData): Promise<TrustPong> {
    const newObj = new TrustPong(sourceId)
    try {
      await newObj._create((cb) => rustAPI().vcx_trust_pong_create_with_request(
        0,
        sourceId,
        request,
        cb
        )
      )
      return newObj
    } catch (err) {
      throw new VCXInternalError(`vcx_trust_pong_create_with_request -> ${err}`)
    }
  }

  static async deserialize (data: ITrustPongData) {
    try {
      const newObj = await super._deserialize<TrustPong, {}>(TrustPong, data)
      return newObj
    } catch (err) {
      throw new VCXInternalError(`vcx_trust_pong_deserialize -> ${err}`)
    }
  }

  static async new_requests (connection: Connection): Promise<ITrustPongRequest[]> {
    const requestsStr = await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_trust_pong_new_pings(0, connection.handle, cb)
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
    const requests = JSON.parse(requestsStr)
    return requests
  }

  async getState (): Promise<StateType> {
    try {
      return await this._getState()
    } catch (error) {
      throw new VCXInternalError(`vcx_trust_pong_get_state -> ${error}`)
    }
  }

  async updateState (): Promise<void> {
    try {
      await this._updateState()
    } catch (error) {
      throw new VCXInternalError(`vcx_trust_pong_update_state -> ${error}`)
    }
  }

  async serialize (): Promise<ITrustPongData> {
    try {
      return JSON.parse(await super._serialize())
    } catch (err) {
      throw new VCXInternalError(`vcx_trust_pong_serialize -> ${err}`)
    }
  }

  async sendPong (connection: Connection): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_trust_pong_send(0, this.handle, connection.handle, cb)
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
      throw new VCXInternalError(`vcx_trust_pong_send -> ${err}`)
    }
  }
}
