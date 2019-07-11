Feature: Mine with a specified plugin
  Scenario: Mine with the cuckatoo_lean_cpu_compat_19 plugin
    Given I define my mining plugin as <cuckatoo_lean_cpu_compat_19>
    Given I choose <cuckoo> algorithm
    Then Mine async for the duration of <20> seconds

  # Scenario: Mine with the cuckatoo_lean_cpu_compat_31 plugin
  #   Given I define my mining plugin as <cuckatoo_lean_cpu_compat_31>
  #   Then Mine async for the duration of <20> seconds

  # Scenario: Mine with the cuckatoo_lean_cpu_avx2_31 plugin
  #   Given I define my mining plugin as <cuckatoo_lean_cpu_avx2_31>
  #   Then Mine async for the duration of <20> seconds

  # Scenario: Mine with the cuckatoo_mean_cpu_compat_19 plugin
  #   Given I define my mining plugin as <cuckatoo_mean_cpu_compat_19>
  #   Then Mine async for the duration of <20> seconds

  # Scenario: Mine with the cuckatoo_mean_cpu_avx2_19 plugin
  #   Given I define my mining plugin as <cuckatoo_mean_cpu_avx2_19>
  #   Then Mine async for the duration of <20> seconds

  Scenario: Miner Randomx
    Given I choose <randomx> algorithm
    Then Mine async for the duration of <20> seconds

  Scenario: Miner progpow
    Given I choose <progpow> algorithm
    Then Mine async for the duration of <20> seconds
