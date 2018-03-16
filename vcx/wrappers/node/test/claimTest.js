const assert = require('chai').assert
const vcx = require('../dist')
const { stubInitVCX } = require('./helpers')

const { Claim, Connection, StateType, Error } = vcx

/* const config = {
  sourceId: 'jsonCreation',
  issuerDid: 'arandomdidfoobar',
  attr: {
    key: 'value',
    key2: 'value2',
    key3: 'value3'
  },
  claimName: 'Claim Name'
} */
/* const formattedAttrs = {
  'account_num': [ '8BEaoLf8TBmK4BUyX8WWnA' ],
  'name_on_account': [ 'Alice' ]
} */
const claimDummyArgs = [
  'Dummy Claim',
  {
    schemaNum: 1,
    issuerDid: 'arandomdidfoobar',
    claimName: 'Claim Name'
  }
]
const message = '{ "msg_type":"CLAIM_OFFER", "version":"0.1", "to_did":"LtMgSjtFcyPwenK9SHCyb8", "from_did":"LtMgSjtFcyPwenK9SHCyb8", "claim":{ "account_num":[ "8BEaoLf8TBmK4BUyX8WWnA" ], "name_on_account":[ "Alice" ] }, "schema_seq_no":48, "issuer_did":"Pd4fnFtRBcMKRVC2go5w3j", "claim_name":"Account Certificate", "claim_id":"3675417066", "msg_ref_id":null }'
describe('An Claim', async function () {
  this.timeout(30000)

  before(async () => {
    stubInitVCX()
    await vcx.initVcx('ENABLE_TEST_MODE')
  })

  it('can be created.', async function () {
    const claim = new Claim(...claimDummyArgs)
    assert(claim)
  })

  it('can have a source Id.', async function () {
    const claim = await new Claim(...claimDummyArgs)
    assert.equal(claim.sourceId, claimDummyArgs[0])
  })

  it('has a state of 0 after instanstiated', async function () {
    const claim = await new Claim(...claimDummyArgs)
    const state = await claim.getState()
    assert.equal(state, 0)
  })

  // it('has a claimHandle and a sourceId after it is created', async function () {
  it('has a sourceId after it is created', async function () {
    const sourceId = 'Claim'
    const claim = await Claim.create_with_message({ sourceId: sourceId, message: message })
    assert.equal(claim.sourceId, sourceId)
  })

  it('has state that can be found', async function () {
    const sourceId = 'TestState'
    const claim = await Claim.create_with_message({ sourceId: sourceId, message: message })
    await claim.updateState()
    assert.equal(await claim.getState(), 3)
  })

  /* it('can be sent with a valid connection', async function () {
    const sourceId = 'Bank Claim'
    let connection = await Connection.create({ id: '234' })
    await connection.connect()
    assert.equal(StateType.OfferSent, await connection.getState())
    const claim = await IssuerClaim.create({ ...config, sourceId })
    await claim.sendOffer(connection)
    await claim.updateState()
    assert.equal(await claim.getState(), StateType.OfferSent)
  }) */

  it('can be created, then serialized, then deserialized and have the same sourceId, state, and claimHandle', async function () {
    const sourceId = 'SerializeDeserialize'
    const claim = await Claim.create_with_message({ sourceId: sourceId, message: message })
    const jsonClaim = await claim.serialize()
    assert.equal(jsonClaim.state, StateType.RequestReceived)
    const claim2 = await Claim.deserialize(jsonClaim)
    // assert.equal(claim.handle, claim2.handle)
    assert.equal(await claim.getState(), await claim2.getState())
  })

  /* it('can be sent, then serialized, then deserialized', async function () {
    // create a connection, send the claim, serialize and then deserialize
    // and compare
    const connection = await Connection.create({ id: '234' })
    await connection.connect()

    const sourceId = 'SendSerializeDeserialize'
    const claim = await IssuerClaim.create({ ...config, sourceId })

    await claim.sendOffer(connection)
    const claimData = await claim.serialize()

    const claim2 = await IssuerClaim.deserialize(claimData)
    await claim.updateState()
    await claim2.updateState()
    assert.equal(await claim.getState(), StateType.OfferSent)
    assert.equal(await claim.getState(), await claim2.getState())
    assert.equal(claim.handle, claim2.handle)
  }) */

  it('serialize without correct handle throws error', async function () {
    const claim = new Claim(null, {})
    try {
      await claim.serialize()
    } catch (error) {
      assert.equal(error.toString(), 'Error: vcx_claim_serialize -> ' + Error.INVALID_ISSUER_CLAIM_HANDLE)
    }
  })

  it('is created from a static method', async function () {
    const sourceId = 'staticMethodCreation'
    const claim = await Claim.create_with_message({ sourceId: sourceId, message: message })
    assert(claim.sourceId, sourceId)
  })

  /* it('will have different claim handles even with the same sourceIds', async function () {
    const sourceId = 'sameSourceIds'
    const claim = await Claim.create_with_message({ sourceId: sourceId, message: message })
    const claim2 = await Claim.create_with_message({ sourceId: sourceId, message: message })
    assert.notEqual(claim.handle, claim2.handle)
  }) */

  it('deserialize is a static method', async function () {
    const sourceId = 'deserializeStatic'
    const claim = await Claim.create_with_message({ sourceId: sourceId, message: message })
    assert.equal(await claim.getState(), StateType.RequestReceived)
    const serializedJson = await claim.serialize()
    const claimDeserialized = await Claim.deserialize(serializedJson)
    assert.equal(await claimDeserialized.getState(), StateType.RequestReceived)
  })

  it('accepts claim attributes and schema sequence number', async function () {
    const sourceId = 'attributesAndSequenceNumber'
    const claim = await Claim.create_with_message({ sourceId: sourceId, message: message })
    assert.equal(claim.sourceId, sourceId)
  })

  it('throws exception for requesting claim with invalid claim handle', async function () {
    let connection = await Connection.create({id: '123'})
    const claim = new Claim(null, {})
    try {
      await claim.sendRequest(connection)
    } catch (error) {
      assert.equal(error.toString(), 'Error: vcx_claim_send_request -> ' + Error.INVALID_ISSUER_CLAIM_HANDLE)
    }
  })

  it('throws exception for requesting claim with invalid connection handle', async function () {
    let releasedConnection = await Connection.create({id: '123'})
    await releasedConnection.release()
    const sourceId = 'Claim'
    const claim = await Claim.create_with_message({ sourceId: sourceId, message: message })
    try {
      await claim.sendRequest(releasedConnection)
    } catch (error) {
      assert.equal(error.toString(), 'Error: vcx_claim_send_request -> ' + Error.INVALID_CONNECTION_HANDLE)
    }
  })

  it('sending claim request with no claim offer should throw exception', async function () {
    const sourceId = 'Claim'
    try {
      await Claim.create_with_message({ sourceId: sourceId, message: null })
    } catch (error) {
      assert.equal(error.toString(), 'Error: vcx_claim_create_with_offer -> ' + Error.INVALID_OPTION)
    }
  })

  // Remove ".skip" once VCX mocks exist
  it.skip('sending claim with valid claim offer should have state VcxStateAccepted', async function () {
    let connection = await Connection.create({id: '123'})
    await connection.connect({ sms: true })
    const sourceId = 'Claim'
    let claim = await Claim.create_with_message({ sourceId: sourceId, message: message })
    await claim.sendRequest(connection)
    assert.equal(await claim.getState(), StateType.OfferSent)
    // we serialize and deserialize because this is the only
    // way to fool the libvcx into thinking we've received a
    // valid claim requset.
    let jsonClaim = await claim.serialize()
    jsonClaim.state = StateType.Accepted
    // jsonClaim.handle += 1
    claim = await Claim.deserialize(jsonClaim)
    assert.equal(await claim.getState(), StateType.Accepted)
  })
})
