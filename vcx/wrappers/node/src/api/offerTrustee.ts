import { Callback } from 'ffi'

import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'
import { StateType } from './common'
import { Connection } from './connection'
import { RecoveryShares } from './recoveryShares'
import { VCXBaseWithState } from './VCXBaseWithState'

export interface IOfferTrusteeData {
  source_id: string,
}

export interface IOfferTrusteeCreateData {
  sourceId: string
}

/**
 * @class Class representing an Issuer Claim
 */
export class OfferTrustee extends VCXBaseWithState {
  protected _releaseFn = rustAPI().vcx_offer_trustee_release
  protected _updateStFn = rustAPI().vcx_offer_trustee_update_state
  protected _getStFn = rustAPI().vcx_offer_trustee_get_state
  protected _serializeFn = rustAPI().vcx_offer_trustee_serialize
  protected _deserializeFn = rustAPI().vcx_offer_trustee_deserialize

  static async create ({ sourceId }: IOfferTrusteeCreateData): Promise<OfferTrustee> {
    const offer = new OfferTrustee(sourceId)
    const commandHandle = 0
    try {
      await offer._create((cb) => rustAPI().vcx_offer_trustee_create(
        commandHandle,
        sourceId,
        cb
        )
      )
      return offer
    } catch (err) {
      throw new VCXInternalError(`vcx_offer_trustee_create -> ${err}`)
    }
  }

  static async deserialize (data: IOfferTrusteeData) {
    try {
      const attr = {}
      const offer = await super._deserialize<OfferTrustee, {}>(OfferTrustee, data, attr)
      return offer
    } catch (err) {
      throw new VCXInternalError(`vcx_offer_trustee_deserialize -> ${err}`)
    }
  }

  /**
   * @memberof OfferTrustee
   * @description Gets the state of the Offer Trustee object.
   * @async
   * @function getState
   * @returns {Promise<StateType>}
   */
  async getState (): Promise<StateType> {
    try {
      return await this._getState()
    } catch (error) {
      throw new VCXInternalError(`vcx_offer_trustee_get_state -> ${error}`)
    }
  }

  /**
   * @memberof OfferTrustee
   * @description Communicates with the agent service for polling and setting the state of the offer trustee.
   * @async
   * @function updateState
   * @returns {Promise<void>}
   */
  async updateState (): Promise<void> {
    try {
      await this._updateState()
    } catch (error) {
      throw new VCXInternalError(`vcx_offer_trustee_update_state -> ${error}`)
    }
  }

  /**
   * @memberof OfferTrustee
   * @description Serializes the object.
   * Data returned can be used to recreate an object by passing it to the deserialize function.
   * @async
   * @function serialize
   * @returns {Promise<IProofData>} - Jason object with all of the underlying Rust attributes.
   * Same json object structure that is passed to the deserialize function.
   */
  async serialize (): Promise<IOfferTrusteeData> {
    try {
      return JSON.parse(await super._serialize())
    } catch (err) {
      throw new VCXInternalError(`vcx_offer_trustee_serialize -> ${err}`)
    }
  }

  /**
   * @memberof OfferTrustee
   * @description Sends a offer to the end user.
   * @async
   * @function sendOffer
   * @param {Connection} connection
   * Connection is the object that was created to set up the pairwise relationship.
   * @returns {Promise<void>}
   */
  async sendOffer (connection: Connection): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
          (resolve, reject, cb) => {
            const rc = rustAPI().vcx_offer_trustee_send_offer(0, this.handle, connection.handle, cb)
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
      throw new VCXInternalError(`vcx_offer_trustee_send_offer -> ${err}`)
    }
  }

/**
 * @memberof OfferTrustee
 * @description Sends the Claim to the end user.
 * Claim is made up of the data sent during Claim Offer
 * @async
 * @function sendClaim
 * @param {Connection} connection
 * Connection is the object that was created to set up the pairwise relationship.
 * @returns {Promise<void>}
 */
  async sendTrusteeData (connection: Connection, recoveryShares: RecoveryShares): Promise<void> {
    try {
      await createFFICallbackPromise<void>(
        (resolve, reject, cb) => {
          const rc = rustAPI().vcx_offer_trustee_send_data(0, this.handle, connection.handle, recoveryShares.handle, cb)
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
      throw new VCXInternalError(`vcx_offer_trustee_send_data -> ${err}`)
    }
  }
}
