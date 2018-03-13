const assert = require('chai').assert
const vcx = require('../dist/index')
const { stubInitVCX } = require('./helpers')
const { RecoveryShares } = vcx

describe('A Shema', function () {
  this.timeout(30000)

  before(async () => {
    stubInitVCX()
    await vcx.initVcx('ENABLE_TEST_MODE')
  })

  it('can be created.', async () => {
    const shares = await RecoveryShares.create('Test', 10, 2)
    assert(shares)
  })

  it('can be serialized.', async () => {
    const shares = await RecoveryShares.create('Test', 10, 2)
    assert(shares)
    const val = await shares.serialize()
    assert(val)
  })

  it('can be deserialized.', async () => {
    const shares = await RecoveryShares.create('Test', 10, 2)
    assert(shares)
    const val = await shares.serialize()
    assert(val)
    const shares2 = await RecoveryShares.deserialize(val)
    assert(shares2)
  })
})
