import { Callback } from 'ffi'

import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { StateType } from './common'
import { Connection } from './connection'
import { VCXBaseWithState } from './VCXBaseWithState'

export interface IDisclosedProofData {
  source_id: string,
}

export class DisclosedProof extends VCXBaseWithState {
  protected _releaseFn = rustAPI().vcx_disclosed_proof_release
  protected _updateStFn = rustAPI().vcx_disclosed_proof_update_state
  protected _getStFn = rustAPI().vcx_disclosed_proof_get_state
  protected _serializeFn = rustAPI().vcx_disclosed_proof_serialize
  protected _deserializeFn = rustAPI().vcx_disclosed_proof_deserialize

  constructor (sourceId, { }) {
    super(sourceId)
  }

  static async create (sourceId: string, request: string): Promise<DisclosedProof> {
    const newObj = new DisclosedProof(sourceId, request)
    try {
      await newObj._create((cb) => rustAPI().vcx_disclosed_proof_create_with_request(
        0,
        sourceId,
        request,
        cb
        )
      )
      return newObj
    } catch (err) {
      throw new VCXInternalError(`vcx_disclosed_proof_create_with_request -> ${err}`)
    }
  }

  static async deserialize (data: IDisclosedProofData) {
    try {
      const attr = {}
      const newObj = await super._deserialize<DisclosedProof, {}>(DisclosedProof, data, attr)
      return newObj
    } catch (err) {
      throw new VCXInternalError(`vcx_disclosed_proof_deserialize -> ${err}`)
    }
  }

  static async new_requests (connection: Connection): Promise<string> {
    return await createFFICallbackPromise<string>(
      (resolve, reject, cb) => {
        const rc = rustAPI().vcx_disclosed_proof_new_requests(0, connection.handle, cb)
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
      throw new VCXInternalError(`vcx_disclosed_proof_get_state -> ${error}`)
    }
  }

  async updateState (): Promise<void> {
    try {
      await this._updateState()
    } catch (error) {
      throw new VCXInternalError(`vcx_disclosed_proof_update_state -> ${error}`)
    }
  }

  async serialize (): Promise<IDisclosedProofData> {
    try {
      return JSON.parse(await super._serialize())
    } catch (err) {
      throw new VCXInternalError(`vcx_disclosed_proof_serialize -> ${err}`)
    }
  }

  async sendProof (connection: Connection): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_disclosed_proof_send_proof(0, this.handle, connection.handle, cb)
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
      throw new VCXInternalError(`vcx_disclosed_proof_send_proof -> ${err}`)
    }
  }
}
