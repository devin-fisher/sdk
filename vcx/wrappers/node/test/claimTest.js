const assert = require('chai').assert
const vcx = require('../dist/index')
const { stubInitVCX } = require('./helpers')
const { Claim } = vcx

describe('A Claim', function () {
  this.timeout(30000)

  const OFFER = {
    msg_type: 'CLAIM_OFFER',
    version: '0.1',
    to_did: 'LtMgSjtFcyPwenK9SHCyb8',
    from_did: 'LtMgSjtFcyPwenK9SHCyb8',
    claim: {
      account_num: [
        '8BEaoLf8TBmK4BUyX8WWnA'
      ],
      name_on_account: [
        'Alice'
      ]
    },
    schema_seq_no: 48,
    issuer_did: 'Pd4fnFtRBcMKRVC2go5w3j',
    claim_name: 'Account Certificate',
    claim_id: '3675417066',
    msg_ref_id: null
  }

  before(async () => {
    stubInitVCX()
    await vcx.initVcx('ENABLE_TEST_MODE')
  })

  it('can be created.', async () => {
    const obj = await Claim.create({sourceId: 'Test', offer: JSON.stringify(OFFER)})
    assert(obj)
  })

  it('can be serialized.', async () => {
    const obj = await Claim.create({sourceId: 'Test', offer: JSON.stringify(OFFER)})
    assert(obj)
    const val = await obj.serialize()
    assert(val)
  })

  it('can be deserialized.', async () => {
    const obj = await Claim.create({sourceId: 'Test', offer: JSON.stringify(OFFER)})
    assert(obj)
    const val = await obj.serialize()
    assert(val)
    const obj2 = await Claim.deserialize(val)
    assert(obj2)
  })

  it('can get state.', async () => {
    const obj = await Claim.create({sourceId: 'Test', offer: JSON.stringify(OFFER)})
    assert(obj)
    const state = await obj.getState()
    console.log(state)
    assert(state === 3)
  })
})
