const assert = require('chai').assert
const vcx = require('../dist/index')
const { stubInitVCX } = require('./helpers')
const { Connection, StateType, Trustee } = vcx

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
    const obj = await Trustee.create({
      sourceId: 'Test',
      offer: OFFER
    })
    assert(obj)
    assert(obj instanceof Trustee)
  })

  it('can be serialized.', async () => {
    const obj = await Trustee.create({
      sourceId: 'Test',
      offer: OFFER
    })
    assert(obj)
    const val = await obj.serialize()
    assert(val)
  })

  it('can be deserialized.', async () => {
    const obj = await Trustee.create({
      sourceId: 'Test',
      offer: OFFER
    })
    assert(obj)
    const val = await obj.serialize()
    assert(val)
    const obj2 = await Trustee.deserialize(val)
    assert(obj2)
    assert(obj2 instanceof Trustee)
  })

  it('can get state.', async () => {
    const obj = await Trustee.create({
      sourceId: 'Test',
      offer: OFFER
    })
    assert(obj)
    const state = await obj.getState()
    console.log(state)
    assert(state === StateType.Received)
  })

  it('can get offers.', async () => {
    const connection = await Connection.create({
      id: '234'
    })
    const offers = await Trustee.new_offers(connection)
    assert(Array.isArray(offers))
  })

  // TODO: Mock having a non-empty array of offers
})
