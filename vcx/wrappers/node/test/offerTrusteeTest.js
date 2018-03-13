const assert = require('chai').assert
const vcx = require('../dist/index')
const { stubInitVCX } = require('./helpers')
const { OfferTrustee } = vcx

describe('A Shema', function () {
  this.timeout(30000)

  before(async () => {
    stubInitVCX()
    await vcx.initVcx('ENABLE_TEST_MODE')
  })

  it('can be created.', async () => {
    const obj = await OfferTrustee.create('Test')
    assert(obj)
  })

  it('can be serialized.', async () => {
    const obj = await OfferTrustee.create('Test')
    assert(obj)
    const val = await obj.serialize()
    assert(val)
  })

  it('can be deserialized.', async () => {
    const obj = await OfferTrustee.create('Test')
    assert(obj)
    const val = await obj.serialize()
    assert(val)
    const obj2 = await OfferTrustee.deserialize(val)
    assert(obj2)
  })

  it('can get state.', async () => {
    const obj = await OfferTrustee.create('Test')
    assert(obj)
    const state = await obj.getState()
    assert(state === 1)
  })
})
