const assert = require('chai').assert
const vcx = require('../dist/index')
const { stubInitVCX } = require('./helpers')
const { Connection, StateType, ReturnShare, VCXMock, Trustee } = vcx

describe('A ReturnShare', function () {
  this.timeout(30000)

  const REQ = {
    version: '0.1',
    msg_type: 'REQUEST_SHARE'
  }

  before(async () => {
    stubInitVCX()
    await vcx.initVcx('ENABLE_TEST_MODE')
  })

  it('can be created.', async () => {
    const obj = await ReturnShare.create({
      sourceId: 'Test',
      request: JSON.stringify(REQ)
    })
    assert(obj)
    assert(obj instanceof ReturnShare)
  })

  it('can be serialized.', async () => {
    const obj = await ReturnShare.create({
      sourceId: 'Test',
      request: JSON.stringify(REQ)
    })
    assert(obj)
    const val = await obj.serialize()
    assert(val)
  })

  it('can be deserialized.', async () => {
    const obj = await ReturnShare.create({
      sourceId: 'Test',
      request: JSON.stringify(REQ)
    })
    assert(obj)
    const val = await obj.serialize()
    assert(val)
    const obj2 = await ReturnShare.deserialize(val)
    assert(obj2)
    assert(obj2 instanceof ReturnShare)
  })

  it('can get state.', async () => {
    const obj = await ReturnShare.create({
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
    const requests = await ReturnShare.new_requests(connection)
    assert(Array.isArray(requests))
  })

  const TRUSTEE_OFFER = {
    version: '0.1',
    msg_type: 'TRUSTEE_OFFER',
    capabilities: ['RECOVERY_SHARE', 'REVOKE_AUTHZ', 'PROVISION_AUTHZ'],
    expires: 1517428815
  }

  it('can get round trip.', async () => {
    const connection = await Connection.create({
      id: '234'
    })
    VCXMock.setVcxMock(18)
    const requests = await ReturnShare.new_requests(connection)
    assert(Array.isArray(requests))
    assert(requests.length > 0)

    const obj = await ReturnShare.create({
      sourceId: 'Test',
      request: JSON.stringify(requests[0])
    })

    assert(obj)
    var state = await obj.getState()
    assert(state === StateType.Received)

    const trustee = await Trustee.create({
      sourceId: 'Test',
      offer: JSON.stringify(TRUSTEE_OFFER)
    }) // NOT FULLY SETUP TRUSTEE BUT TEST MODE WILL ALLOW THIS TO WORK

    await obj.sendShare(connection, trustee)
    state = await obj.getState()
    assert(state === StateType.Accepted)
  })
})
