import { Callback } from 'ffi'

import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { StateType } from './common'
import { Connection } from './connection'
import { Trustee } from './trustee'
import { VCXBaseWithState } from './VCXBaseWithState'

export interface IReturnShareData {
  source_id: string,
}

export type IReturnShareRequest = string

export interface IReturnShareCreateData {
  sourceId: string,
  request: IReturnShareRequest
}

export class ReturnShare extends VCXBaseWithState {
  protected _releaseFn = rustAPI().vcx_return_share_release
  protected _updateStFn = rustAPI().vcx_return_share_update_state
  protected _getStFn = rustAPI().vcx_return_share_get_state
  protected _serializeFn = rustAPI().vcx_return_share_serialize
  protected _deserializeFn = rustAPI().vcx_return_share_deserialize

  static async create ({ sourceId, request }: IReturnShareCreateData): Promise<ReturnShare> {
    const newObj = new ReturnShare(sourceId)
    try {
      await newObj._create((cb) => rustAPI().vcx_return_share_create_with_request(
        0,
        sourceId,
        request,
        cb
        )
      )
      return newObj
    } catch (err) {
      throw new VCXInternalError(`vcx_return_share_create_with_request -> ${err}`)
    }
  }

  static async deserialize (data: IReturnShareData) {
    try {
      const newObj = await super._deserialize<ReturnShare, {}>(ReturnShare, data)
      return newObj
    } catch (err) {
      throw new VCXInternalError(`vcx_return_share_deserialize -> ${err}`)
    }
  }

  static async new_requests (connection: Connection): Promise<IReturnShareRequest[]> {
    const requestsStr = await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_return_share_new_request(0, connection.handle, cb)
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
      throw new VCXInternalError(`vcx_return_share_get_state -> ${error}`)
    }
  }

  async updateState (): Promise<void> {
    try {
      await this._updateState()
    } catch (error) {
      throw new VCXInternalError(`vcx_return_share_update_state -> ${error}`)
    }
  }

  async serialize (): Promise<IReturnShareData> {
    try {
      return JSON.parse(await super._serialize())
    } catch (err) {
      throw new VCXInternalError(`vcx_return_share_serialize -> ${err}`)
    }
  }

  async sendShare (connection: Connection, trustee: Trustee): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_return_share_send_share(0, this.handle, connection.handle, trustee.handle, cb)
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
      throw new VCXInternalError(`vcx_return_share_send_share -> ${err}`)
    }
  }
}
