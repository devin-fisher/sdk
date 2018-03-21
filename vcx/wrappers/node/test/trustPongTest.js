const assert = require('chai').assert
const vcx = require('../dist/index')
const { stubInitVCX } = require('./helpers')
const { Connection, StateType, TrustPong } = vcx

describe('A TrustPong', function () {
  this.timeout(30000)

  const REQ = {
    nonce: '184470606884409637577315',
    msg_type: 'TRUST_PING'
  }

  before(async () => {
    stubInitVCX()
    await vcx.initVcx('ENABLE_TEST_MODE')
  })

  it('can be created.', async () => {
    const obj = await TrustPong.create({
      sourceId: 'Test',
      request: JSON.stringify(REQ)
    })
    assert(obj)
    assert(obj instanceof TrustPong)
  })

  it('can be serialized.', async () => {
    const obj = await TrustPong.create({
      sourceId: 'Test',
      request: JSON.stringify(REQ)
    })
    assert(obj)
    const val = await obj.serialize()
    assert(val)
  })

  it('can be deserialized.', async () => {
    const obj = await TrustPong.create({
      sourceId: 'Test',
      request: JSON.stringify(REQ)
    })
    assert(obj)
    const val = await obj.serialize()
    assert(val)
    const obj2 = await TrustPong.deserialize(val)
    assert(obj2)
    assert(obj2 instanceof TrustPong)
  })

  it('can get state.', async () => {
    const obj = await TrustPong.create({
      sourceId: 'Test',
      request: JSON.stringify(REQ)
    })
    assert(obj)
    const state = await obj.getState()
    assert(state === StateType.Received)
  })

  it('can get requests.', async () => {
    const connection = await Connection.create({
      id: '234'
    })
    const requests = await TrustPong.new_requests(connection)
    assert(Array.isArray(requests))
  })

  // TODO: Mock having a non-empty array of requests
})
