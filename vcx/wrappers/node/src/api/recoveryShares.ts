import { _ } from 'lodash'
import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { VCXBase } from './VCXBase'

export interface IRecoveryShares {
  source_id: string,
}

export class RecoveryShares extends VCXBase {
  protected _releaseFn = rustAPI().vcx_recovery_shares_release
  protected _serializeFn = rustAPI().vcx_recovery_shares_serialize
  protected _deserializeFn = rustAPI().vcx_recovery_shares_deserialize

  constructor (sourceId) {
    super(sourceId)
  }

  static async create (sourceId: string, count: number, threshold: number): Promise<RecoveryShares> {
    const shares = new RecoveryShares(sourceId)
    const commandHandle = 0
    try {
      await shares._create((cb) => rustAPI().vcx_recovery_shares_create(
      commandHandle,
      sourceId,
      count,
      threshold,
      cb
      ))
      return shares
    } catch (err) {
      throw new VCXInternalError(`vcx_recovery_shares_create -> ${err}`)
    }
  }

  static async deserialize (data: IRecoveryShares) {
    try {
      const newObj = await super._deserialize<RecoveryShares, {}>(RecoveryShares, data)
      return newObj
    } catch (err) {
      throw new VCXInternalError(`vcx_recovery_shares_serialize -> ${err}`)
    }
  }

  async serialize (): Promise<IRecoveryShares> {
    try {
      const data: IRecoveryShares = JSON.parse(await super._serialize())
      return data
    } catch (err) {
      throw new VCXInternalError(`vcx_recovery_shares_deserialize -> ${err}`)
    }
  }
}
