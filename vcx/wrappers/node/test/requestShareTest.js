const assert = require('chai').assert
const vcx = require('../dist/index')
const { stubInitVCX } = require('./helpers')
const { RequestShare, StateType, Connection, VCXMock } = vcx

describe('RequestShare', function () {
  this.timeout(30000)

  before(async () => {
    stubInitVCX()
    await vcx.initVcx('ENABLE_TEST_MODE')
  })

  it('can be created.', async () => {
    const obj = await RequestShare.create({ sourceId: 'Test' })
    assert(obj)
    assert(obj instanceof RequestShare)
  })

  it('can be serialized.', async () => {
    const obj = await RequestShare.create({ sourceId: 'Test' })
    assert(obj)
    const val = await obj.serialize()
    assert(val)
  })

  it('can be deserialized.', async () => {
    const obj = await RequestShare.create({ sourceId: 'Test' })
    assert(obj)
    const val = await obj.serialize()
    assert(val)
    const obj2 = await RequestShare.deserialize(val)
    assert(obj2)
    assert(obj2 instanceof RequestShare)
  })

  it('can get state.', async () => {
    const obj = await RequestShare.create({ sourceId: 'Test' })
    assert(obj)
    const state = await obj.getState()
    assert(state === StateType.Initialized)
  })

  it('can round trip.', async () => {
    const obj = await RequestShare.create({ sourceId: 'Test' })
    assert(obj)
    var state = await obj.getState()
    assert(state === StateType.Initialized)

    let connection = await Connection.create({ id: '234' })
    await connection.connect()

    obj.requestShare(connection)
    await obj.updateState()
    state = await obj.getState()
    assert(state === StateType.Sent)

    VCXMock.setVcxMock(20)
    VCXMock.setVcxMock(19)

    obj.updateState()
    state = await obj.getState()
    assert(state === StateType.Accepted)
  })
})
