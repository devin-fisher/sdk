import { Callback } from 'ffi'

import { VCXInternalError } from '../errors'
import { rustAPI } from '../rustlib'
import { createFFICallbackPromise } from '../utils/ffi-helpers'

export async function backup (fileList: string[]): Promise<void> {
  let rc = 0
  const fileListJson = JSON.stringify(fileList)
  try {
    return await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        rc = rustAPI().vcx_backup_do_backup(0, fileListJson, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback('void', ['uint32', 'uint32'], (xhandle, err) => {
        if (err) {
          reject(err)
        } else {
          resolve()
        }
      })
    )
  } catch (err) {
    throw new VCXInternalError(`vcx_backup_do_backup -> ${err}`)
  }
}

export async function restore (shareHandles: string[]): Promise<void> {
  let rc = 0
  const shareHandlesJson = JSON.stringify(shareHandles)
  try {
    return await createFFICallbackPromise<void>(
      (resolve, reject, cb) => {
        rc = rustAPI().vcx_backup_do_restore(0, shareHandlesJson, cb)
        if (rc) {
          reject(rc)
        }
      },
      (resolve, reject) => Callback('void', ['uint32', 'uint32'], (xhandle, err) => {
        if (err) {
          reject(err)
        } else {
          resolve()
        }
      })
    )
  } catch (err) {
    throw new VCXInternalError(`vcx_backup_do_restore -> ${err}`)
  }
}
