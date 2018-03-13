const assert = require('chai').assert
const vcx = require('../dist/index')
const { stubInitVCX } = require('./helpers')
const { Trustee } = vcx

describe('A Trustee', function () {
  this.timeout(30000)

  const OFFER = {
    version: '0.1',
    msg_type: 'TRUSTEE_OFFER',
    capabilities: ['RECOVERY_SHARE', 'REVOKE_AUTHZ', 'PROVISION_AUTHZ'],
    expires: 1517428815
  }

  before(async () => {
    stubInitVCX()
    await vcx.initVcx('ENABLE_TEST_MODE')
  })

  it('can be created.', async () => {
    const obj = await Trustee.create('Test', JSON.stringify(OFFER))
    assert(obj)
  })

  it('can be serialized.', async () => {
    const obj = await Trustee.create('Test', JSON.stringify(OFFER))
    assert(obj)
    const val = await obj.serialize()
    assert(val)
  })

  it('can be deserialized.', async () => {
    const obj = await Trustee.create('Test', JSON.stringify(OFFER))
    assert(obj)
    const val = await obj.serialize()
    assert(val)
    const obj2 = await Trustee.deserialize(val)
    assert(obj2)
  })

  it('can get state.', async () => {
    const obj = await Trustee.create('Test', JSON.stringify(OFFER))
    assert(obj)
    const state = await obj.getState()
    console.log(state)
    assert(state === 3)
  })
})
