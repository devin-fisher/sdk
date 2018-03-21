import { Callback } from 'ffi'
import { _ } from 'lodash'
import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { StateType } from './common'
import { Connection } from './connection'
import { VCXBaseWithState } from './VCXBaseWithState'

export interface IRequestShareConfig {
  sourceId: string,
}

/**
 */
export interface IRequestShareData {
  source_id: string
}

/**
 */
export class RequestShare extends VCXBaseWithState {
  protected _releaseFn = rustAPI().vcx_request_share_release
  protected _updateStFn = rustAPI().vcx_request_share_update_state
  protected _getStFn = rustAPI().vcx_request_share_get_state
  protected _serializeFn = rustAPI().vcx_request_share_serialize
  protected _deserializeFn = rustAPI().vcx_request_share_deserialize

  /**
   */
  static async create (data: IRequestShareConfig): Promise<RequestShare> {
    const obj = new RequestShare(data.sourceId)

    try {
      await obj._create((cb) => rustAPI().vcx_request_share_create(
        0,
        obj.sourceId,
        cb
      ))
      return obj
    } catch (err) {
      throw new VCXInternalError(`vcx_request_share_create -> ${err}`)
    }
  }

  /**
   */
  static async deserialize (data: IRequestShareData) {
    try {
      const obj = await super._deserialize(RequestShare, data)
      return obj
    } catch (err) {
      throw new VCXInternalError(`vcx_request_share_deserialize -> ${err}`)
    }
  }

  /**
   */
  async serialize (): Promise<IRequestShareData> {
    try {
      const data: IRequestShareData = JSON.parse(await super._serialize())
      return data
    } catch (err) {
      throw new VCXInternalError(`vcx_request_share_serialize -> ${err}`)
    }
  }

  /**
   */
  async getState (): Promise<StateType> {
    try {
      return await this._getState()
    } catch (error) {
      throw new VCXInternalError(`vcx_request_share_get_state -> ${error}`)
    }
  }

  /**
   */
  async updateState (): Promise<void> {
    try {
      await this._updateState()
    } catch (error) {
      throw new VCXInternalError(`vcx_request_share_update_state -> ${error}`)
    }
  }

  /**
   */
  async requestShare (connection: Connection): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_request_share_send_request(0, this.handle, connection.handle, cb)
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
      throw new VCXInternalError(`vcx_request_share_send_request -> ${err}`)
    }
  }
}
