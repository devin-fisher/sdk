const assert = require('chai').assert
const vcx = require('../dist/index')
const { stubInitVCX } = require('./helpers')
const { OfferTrustee, StateType } = vcx

describe('OfferTrustee', function () {
  this.timeout(30000)

  before(async () => {
    stubInitVCX()
    await vcx.initVcx('ENABLE_TEST_MODE')
  })

  it('can be created.', async () => {
    const obj = await OfferTrustee.create({ sourceId: 'Test' })
    assert(obj)
    assert(obj instanceof OfferTrustee)
  })

  it('can be serialized.', async () => {
    const obj = await OfferTrustee.create({ sourceId: 'Test' })
    assert(obj)
    const val = await obj.serialize()
    assert(val)
  })

  it('can be deserialized.', async () => {
    const obj = await OfferTrustee.create({ sourceId: 'Test' })
    assert(obj)
    const val = await obj.serialize()
    assert(val)
    const obj2 = await OfferTrustee.deserialize(val)
    assert(obj2)
    assert(obj2 instanceof OfferTrustee)
  })

  it('can get state.', async () => {
    const obj = await OfferTrustee.create({ sourceId: 'Test' })
    assert(obj)
    const state = await obj.getState()
    assert(state === StateType.Initialized)
  })
})
