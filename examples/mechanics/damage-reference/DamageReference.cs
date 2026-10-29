// ORIGINAL REFERENCE MODEL -- independently authored numeric diagnostic.
// This is not recovered or decompiled game source and is not a player calculator.
// It models only caller-supplied scalars and controlled callback boundaries.

using System;

namespace DamageReference;

public readonly record struct FinalizationTrace(
    double Amount,
    int OldSeed,
    int NewSeed,
    int Upper,
    int Lower,
    int Remainder,
    int Weighted,
    float Fluctuation,
    float ExtraRate,
    double PromotedExtraRate,
    double AfterExtraRate,
    double BeforeCeiling,
    double Rounded,
    double Result);

public static class DamageFinalization
{
    public static FinalizationTrace FinalizeAmount(
        double amount,
        ref int seed,
        Func<int, int> advanceSeed,
        Func<int> readUpper,
        Func<int> readLower,
        float extraRate,
        bool negateAfterCeiling)
    {
        int oldSeed = seed;
        int newSeed = advanceSeed(oldSeed);
        seed = newSeed;
        int upper = readUpper();
        int lower = readLower();
        int remainder = oldSeed % 10000;

        int weighted;
        unchecked
        {
            weighted = (upper - lower) * remainder + lower;
        }

        float fluctuation = (float)weighted * 0.0001f;
        double promotedExtraRate = (double)extraRate;
        double afterExtraRate = amount * promotedExtraRate;
        double beforeCeiling = afterExtraRate * (double)fluctuation;
        double rounded = Math.Ceiling(beforeCeiling);
        double result = negateAfterCeiling ? 0.0 - rounded : rounded;

        return new FinalizationTrace(
            amount,
            oldSeed,
            newSeed,
            upper,
            lower,
            remainder,
            weighted,
            fluctuation,
            extraRate,
            promotedExtraRate,
            afterExtraRate,
            beforeCeiling,
            rounded,
            result);
    }
}
