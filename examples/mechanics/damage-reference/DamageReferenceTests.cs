// ORIGINAL REFERENCE MODEL TEST RUNNER -- authored numeric vectors only.
// These assertions exercise the independent model, not recovered game code.
// Expected values are independent literals or hardcoded IEEE-754 bit patterns.
// BCL-only. Console is the only I/O; no game, process, network, or file access.

using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

namespace DamageReference;

internal static class DamageReferenceTests
{
    private static int _passed;
    private static int _failed;
    private static int _assertions;

    private static readonly (string Name, Action Body)[] Tests =
    {
        ("float factor boundary", FloatFactorBoundary),
        ("left-to-right double products", LeftToRightProducts),
        ("old seed and signed remainder", OldSeedAndSignedRemainder),
        ("callback order and captured trace", CallbackOrderAndCapturedTrace),
        ("callback exceptions preserve prior mutation", CallbackExceptionsPreserveMutation),
        ("unchecked integer weighting", UncheckedIntegerWeighting),
        ("ceiling neighbors", CeilingNeighbors),
        ("double amount and negative ceiling", DoubleAmountAndNegativeCeiling),
        ("negation follows ceiling", NegationFollowsCeiling),
        ("non-finite values are not sanitized", NonFiniteValuesAreNotSanitized),
    };

    public static int Main()
    {
        Console.WriteLine("DamageReference authored numeric diagnostic");
        Console.WriteLine($"RUNTIME: {RuntimeInformation.FrameworkDescription}; Environment.Version={Environment.Version}");
        foreach ((string name, Action body) in Tests)
        {
            Run(name, body);
        }

        Console.WriteLine($"TOTAL: passed={_passed} failed={_failed} assertions={_assertions}");
        return _failed == 0 ? 0 : 1;
    }

    private static void Run(string name, Action body)
    {
        int before = _assertions;
        try
        {
            body();
            _passed++;
            Console.WriteLine($"PASS {name} assertions={_assertions - before}");
        }
        catch (Exception exception)
        {
            _failed++;
            Console.WriteLine($"FAIL {name}: {exception.GetType().Name}: {exception.Message}");
        }
    }

    private static void FloatFactorBoundary()
    {
        int seed = 2;
        FinalizationTrace trace = DamageFinalization.FinalizeAmount(
            10000.0001,
            ref seed,
            _ => 0,
            () => 1,
            () => 1,
            1f,
            false);

        EqualDoubleBits("Amount", 0x40C388000346DC5DUL, trace.Amount);
        EqualInt("OldSeed", 2, trace.OldSeed);
        EqualInt("NewSeed", 0, trace.NewSeed);
        EqualInt("Upper", 1, trace.Upper);
        EqualInt("Lower", 1, trace.Lower);
        EqualInt("Remainder", 2, trace.Remainder);
        EqualInt("Weighted", 1, trace.Weighted);
        EqualFloatBits("Fluctuation", 0x38D1B717U, trace.Fluctuation);
        EqualFloatBits("ExtraRate", 0x3F800000U, trace.ExtraRate);
        EqualDoubleBits("PromotedExtraRate", 0x3FF0000000000000UL, trace.PromotedExtraRate);
        EqualDoubleBits("AfterExtraRate", 0x40C388000346DC5DUL, trace.AfterExtraRate);
        EqualDoubleBits("BeforeCeiling", 0x3FEFFFFFF7CE63B6UL, trace.BeforeCeiling);
        EqualDoubleBits("Rounded", 0x3FF0000000000000UL, trace.Rounded);
        EqualDoubleBits("Result", 0x3FF0000000000000UL, trace.Result);
        EqualInt("mutated seed", 0, seed);

        double wrongDoubleFactor = 10000.0001 * 0.0001;
        EqualDoubleBits("double-factor alternative", 0x3FF0000002AF31DCUL, wrongDoubleFactor);
        EqualDoubleBits("wrong double-factor ceiling", 0x4000000000000000UL, Math.Ceiling(wrongDoubleFactor));
    }

    private static void LeftToRightProducts()
    {
        double amount = BitConverter.Int64BitsToDouble(unchecked((long)0x401AAAAAA4000003UL));
        float extraRate = BitConverter.Int32BitsToSingle(unchecked((int)0x3DCCCCCDU));
        int seed = 2;
        FinalizationTrace trace = DamageFinalization.FinalizeAmount(
            amount,
            ref seed,
            _ => 0,
            () => 15000,
            () => 15000,
            extraRate,
            false);

        EqualDoubleBits("Amount", 0x401AAAAAA4000003UL, trace.Amount);
        EqualInt("OldSeed", 2, trace.OldSeed);
        EqualInt("NewSeed", 0, trace.NewSeed);
        EqualInt("Upper", 15000, trace.Upper);
        EqualInt("Lower", 15000, trace.Lower);
        EqualInt("Remainder", 2, trace.Remainder);
        EqualInt("Weighted", 15000, trace.Weighted);
        EqualFloatBits("Fluctuation", 0x3FC00000U, trace.Fluctuation);
        EqualFloatBits("ExtraRate", 0x3DCCCCCDU, trace.ExtraRate);
        EqualDoubleBits("PromotedExtraRate", 0x3FB99999A0000000UL, trace.PromotedExtraRate);
        EqualDoubleBits("AfterExtraRate", 0x3FE5555555555556UL, trace.AfterExtraRate);
        EqualDoubleBits("BeforeCeiling", 0x3FF0000000000000UL, trace.BeforeCeiling);
        EqualDoubleBits("Rounded", 0x3FF0000000000000UL, trace.Rounded);
        EqualDoubleBits("Result", 0x3FF0000000000000UL, trace.Result);

        double incorrectlyGrouped = amount * ((double)extraRate * 1.5);
        EqualDoubleBits("grouped alternative", 0x3FF0000000000001UL, incorrectlyGrouped);
        EqualDoubleBits("grouped alternative ceiling", 0x4000000000000000UL, Math.Ceiling(incorrectlyGrouped));
    }

    private static void OldSeedAndSignedRemainder()
    {
        int positiveSeed = 2;
        FinalizationTrace positive = DamageFinalization.FinalizeAmount(
            1.0,
            ref positiveSeed,
            _ => 0,
            () => 10001,
            () => 10000,
            1f,
            false);
        EqualInt("positive old seed", 2, positive.OldSeed);
        EqualInt("positive new seed", 0, positive.NewSeed);
        EqualInt("old-seed remainder", 2, positive.Remainder);
        EqualInt("old-seed weighted", 10002, positive.Weighted);
        EqualFloatBits("old-seed fluctuation", 0x3F80068EU, positive.Fluctuation);
        EqualDoubleBits("new seed must not drive remainder", 0x4000000000000000UL, positive.Result);
        EqualInt("stored positive next seed", 0, positiveSeed);

        int negativeSeed = -1;
        FinalizationTrace negative = DamageFinalization.FinalizeAmount(
            1.0,
            ref negativeSeed,
            _ => 7,
            () => 10001,
            () => 10000,
            1f,
            false);
        EqualInt("negative old seed", -1, negative.OldSeed);
        EqualInt("negative new seed", 7, negative.NewSeed);
        EqualInt("signed remainder", -1, negative.Remainder);
        EqualInt("signed weighted", 9999, negative.Weighted);
        EqualFloatBits("signed fluctuation", 0x3F7FF972U, negative.Fluctuation);
        EqualDoubleBits("signed result", 0x3FF0000000000000UL, negative.Result);
        EqualInt("stored negative next seed", 7, negativeSeed);

        int negativeLargeSeed = -10001;
        FinalizationTrace negativeLarge = DamageFinalization.FinalizeAmount(
            1.0,
            ref negativeLargeSeed,
            _ => 7,
            () => 10001,
            () => 10000,
            1f,
            false);
        EqualInt("negative large old seed", -10001, negativeLarge.OldSeed);
        EqualInt("negative large next seed", 7, negativeLarge.NewSeed);
        EqualInt("negative large signed remainder", -1, negativeLarge.Remainder);
        EqualInt("negative large weighted", 9999, negativeLarge.Weighted);
        EqualFloatBits("negative large fluctuation", 0x3F7FF972U, negativeLarge.Fluctuation);
        EqualDoubleBits("negative large result", 0x3FF0000000000000UL, negativeLarge.Result);
        EqualInt("stored negative large next seed", 7, negativeLargeSeed);

        int zeroSeed = 0;
        FinalizationTrace zero = DamageFinalization.FinalizeAmount(
            1.0,
            ref zeroSeed,
            _ => 7,
            () => 10001,
            () => 10000,
            1f,
            false);
        EqualInt("zero old seed", 0, zero.OldSeed);
        EqualInt("zero next seed", 7, zero.NewSeed);
        EqualInt("zero old-seed remainder", 0, zero.Remainder);
        EqualInt("zero old-seed weighted", 10000, zero.Weighted);
        EqualFloatBits("zero old-seed fluctuation", 0x3F800000U, zero.Fluctuation);
        EqualDoubleBits("zero old seed must drive result", 0x3FF0000000000000UL, zero.Result);
        EqualInt("stored zero-case next seed", 7, zeroSeed);
    }

    private static void CallbackOrderAndCapturedTrace()
    {
        var events = new List<string>();
        int seed = 2;
        int advanceCalls = 0;
        int upperCalls = 0;
        int lowerCalls = 0;
        FinalizationTrace trace = DamageFinalization.FinalizeAmount(
            1.0,
            ref seed,
            oldSeed =>
            {
                advanceCalls++;
                events.Add($"advance({oldSeed})");
                return 0;
            },
            () =>
            {
                upperCalls++;
                events.Add($"upper(seed={seed})");
                return 10001;
            },
            () =>
            {
                lowerCalls++;
                events.Add($"lower(seed={seed})");
                return 10000;
            },
            1f,
            false);

        EqualString("callback sequence", "advance(2);upper(seed=0);lower(seed=0)", string.Join(";", events));
        EqualInt("advance calls", 1, advanceCalls);
        EqualInt("upper calls", 1, upperCalls);
        EqualInt("lower calls", 1, lowerCalls);
        EqualInt("seed visible to bounds", 0, seed);

        string capturedText = trace.ToString();
        _ = trace.Amount;
        _ = trace.Fluctuation;
        _ = trace.Result;
        EqualString("repeat trace formatting", capturedText, trace.ToString());
        EqualInt("reads do not replay advance", 1, advanceCalls);
        EqualInt("reads do not replay upper", 1, upperCalls);
        EqualInt("reads do not replay lower", 1, lowerCalls);
    }

    private static void CallbackExceptionsPreserveMutation()
    {
        int providerSeed = 2;
        var providerEvents = new List<string>();
        Exception? providerError = null;
        try
        {
            DamageFinalization.FinalizeAmount(
                1.0,
                ref providerSeed,
                oldSeed =>
                {
                    providerEvents.Add($"advance({oldSeed})");
                    throw new InvalidOperationException("provider");
                },
                () =>
                {
                    providerEvents.Add("upper");
                    return 10000;
                },
                () =>
                {
                    providerEvents.Add("lower");
                    return 10000;
                },
                1f,
                false);
        }
        catch (Exception exception)
        {
            providerError = exception;
        }
        True("provider exception type", providerError is InvalidOperationException);
        EqualInt("provider throw leaves seed", 2, providerSeed);
        EqualString("provider throw skips bounds", "advance(2)", string.Join(";", providerEvents));

        int upperSeed = 2;
        var upperEvents = new List<string>();
        Exception? upperError = null;
        try
        {
            DamageFinalization.FinalizeAmount(
                1.0,
                ref upperSeed,
                oldSeed =>
                {
                    upperEvents.Add($"advance({oldSeed})");
                    return 0;
                },
                () =>
                {
                    upperEvents.Add($"upper(seed={upperSeed})");
                    throw new InvalidOperationException("upper");
                },
                () =>
                {
                    upperEvents.Add("lower");
                    return 10000;
                },
                1f,
                false);
        }
        catch (Exception exception)
        {
            upperError = exception;
        }
        True("upper exception type", upperError is InvalidOperationException);
        EqualInt("upper throw keeps stored next seed", 0, upperSeed);
        EqualString("upper throw skips lower", "advance(2);upper(seed=0)", string.Join(";", upperEvents));

        int lowerSeed = 2;
        var lowerEvents = new List<string>();
        Exception? lowerError = null;
        try
        {
            DamageFinalization.FinalizeAmount(
                1.0,
                ref lowerSeed,
                oldSeed =>
                {
                    lowerEvents.Add($"advance({oldSeed})");
                    return 0;
                },
                () =>
                {
                    lowerEvents.Add($"upper(seed={lowerSeed})");
                    return 10000;
                },
                () =>
                {
                    lowerEvents.Add($"lower(seed={lowerSeed})");
                    throw new InvalidOperationException("lower");
                },
                1f,
                false);
        }
        catch (Exception exception)
        {
            lowerError = exception;
        }
        True("lower exception type", lowerError is InvalidOperationException);
        EqualInt("lower throw keeps stored next seed", 0, lowerSeed);
        EqualString("lower throw order", "advance(2);upper(seed=0);lower(seed=0)", string.Join(";", lowerEvents));
    }

    private static void UncheckedIntegerWeighting()
    {
        int seed = 2;
        FinalizationTrace trace = DamageFinalization.FinalizeAmount(
            1.0,
            ref seed,
            _ => 0,
            () => int.MaxValue,
            () => int.MinValue,
            1f,
            false);

        EqualInt("wrapped weighted integer", 2147483646, trace.Weighted);
        EqualFloatBits("wrapped fluctuation", 0x4851B717U, trace.Fluctuation);
        EqualDoubleBits("wrapped pre-ceiling", 0x410A36E2E0000000UL, trace.BeforeCeiling);
        EqualDoubleBits("wrapped result", 0x410A36E800000000UL, trace.Result);
    }

    private static void CeilingNeighbors()
    {
        FinalizationTrace below = Uniform(Math.BitDecrement(1.0));
        FinalizationTrace exact = Uniform(1.0);
        FinalizationTrace above = Uniform(Math.BitIncrement(1.0));

        EqualDoubleBits("BitDecrement(1) ceiling", 0x3FF0000000000000UL, below.Result);
        EqualDoubleBits("exact 1 ceiling", 0x3FF0000000000000UL, exact.Result);
        EqualDoubleBits("BitIncrement(1) ceiling", 0x4000000000000000UL, above.Result);
    }

    private static void DoubleAmountAndNegativeCeiling()
    {
        FinalizationTrace wide = Uniform(16777217.0);
        EqualDoubleBits("double amount retained", 0x4170000010000000UL, wide.Amount);
        EqualDoubleBits("no premature float narrowing", 0x4170000010000000UL, wide.Result);

        FinalizationTrace negative = Uniform(-1.25);
        EqualDoubleBits("negative amount retained", 0xBFF4000000000000UL, negative.Amount);
        EqualDoubleBits("Ceiling(-1.25), not floor or clamp", 0xBFF0000000000000UL, negative.Result);
    }

    private static void NegationFollowsCeiling()
    {
        FinalizationTrace trace = Uniform(3.0, 0.5f, true);

        EqualDoubleBits("before ceiling", 0x3FF8000000000000UL, trace.BeforeCeiling);
        EqualDoubleBits("ceiling before sign", 0x4000000000000000UL, trace.Rounded);
        EqualDoubleBits("post-ceiling subtraction", 0xC000000000000000UL, trace.Result);
        EqualDoubleBits("ceil after sign is a different result", 0xBFF0000000000000UL, Math.Ceiling(-1.5));
    }

    private static void NonFiniteValuesAreNotSanitized()
    {
        FinalizationTrace nan = Uniform(double.NaN);
        True("NaN amount", double.IsNaN(nan.Amount));
        True("NaN survives multiplication", double.IsNaN(nan.BeforeCeiling));
        True("NaN survives ceiling", double.IsNaN(nan.Rounded) && double.IsNaN(nan.Result));
        EqualInt("NaN still advances seed", 0, nan.NewSeed);

        FinalizationTrace infinity = Uniform(double.PositiveInfinity);
        True("infinity survives multiplication", double.IsPositiveInfinity(infinity.BeforeCeiling));
        True("infinity survives ceiling", double.IsPositiveInfinity(infinity.Rounded) && double.IsPositiveInfinity(infinity.Result));
        EqualInt("infinity still advances seed", 0, infinity.NewSeed);
    }

    private static FinalizationTrace Uniform(double amount, float extraRate = 1f, bool negateAfterCeiling = false)
    {
        int seed = 2;
        return DamageFinalization.FinalizeAmount(
            amount,
            ref seed,
            _ => 0,
            () => 10000,
            () => 10000,
            extraRate,
            negateAfterCeiling);
    }

    private static void EqualInt(string name, int expected, int actual)
    {
        _assertions++;
        if (expected != actual)
        {
            throw new InvalidOperationException($"{name}: expected {expected}, got {actual}");
        }
    }

    private static void EqualFloatBits(string name, uint expected, float actual)
    {
        _assertions++;
        uint actualBits = unchecked((uint)BitConverter.SingleToInt32Bits(actual));
        if (expected != actualBits)
        {
            throw new InvalidOperationException($"{name}: expected 0x{expected:X8}, got 0x{actualBits:X8}");
        }
    }

    private static void EqualDoubleBits(string name, ulong expected, double actual)
    {
        _assertions++;
        ulong actualBits = unchecked((ulong)BitConverter.DoubleToInt64Bits(actual));
        if (expected != actualBits)
        {
            throw new InvalidOperationException($"{name}: expected 0x{expected:X16}, got 0x{actualBits:X16}");
        }
    }

    private static void EqualString(string name, string expected, string actual)
    {
        _assertions++;
        if (!string.Equals(expected, actual, StringComparison.Ordinal))
        {
            throw new InvalidOperationException($"{name}: expected '{expected}', got '{actual}'");
        }
    }

    private static void True(string name, bool actual)
    {
        _assertions++;
        if (!actual)
        {
            throw new InvalidOperationException($"{name}: expected true");
        }
    }
}
