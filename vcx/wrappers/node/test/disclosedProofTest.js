const assert = require('chai').assert
const vcx = require('../dist/index')
const { stubInitVCX } = require('./helpers')
const { DisclosedProof } = vcx

describe('A Claim', function () {
  this.timeout(30000)
  const REQ = `{
    "@type":{
      "name":"PROOF_REQUEST",
      "version":"1.0"
    },
    "@topic":{
      "mid":9,
      "tid":1
    },
    "proof_request_data":{
      "nonce":"838186471541979035208225",
      "name":"Account Certificate",
      "version":"0.1",
      "requested_attrs":{
        "email_1":{
          "name":"email",
          "schema_seq_no":52
        },
        "business_2":{
          "name":"business",
          "schema_seq_no":52
        },
        "name_0":{
          "name":"name",
          "schema_seq_no":52
        }
      },
      "requested_predicates":{
  
      }
    },
    "msg_ref_id":null
  }`

  before(async () => {
    stubInitVCX()
    await vcx.initVcx('ENABLE_TEST_MODE')
  })

  it('can be created.', async () => {
    const obj = await DisclosedProof.create('Test', REQ)
    assert(obj)
  })

  it('can be serialized.', async () => {
    const obj = await DisclosedProof.create('Test', REQ)
    assert(obj)
    const val = await obj.serialize()
    assert(val)
  })

  it('can be deserialized.', async () => {
    const obj = await DisclosedProof.create('Test', REQ)
    assert(obj)
    const val = await obj.serialize()
    assert(val)
    const obj2 = await DisclosedProof.deserialize(val)
    assert(obj2)
  })

  it('can get state.', async () => {
    const obj = await DisclosedProof.create('Test', REQ)
    assert(obj)
    const state = await obj.getState()
    console.log(state)
    assert(state === 3)
  })
})
