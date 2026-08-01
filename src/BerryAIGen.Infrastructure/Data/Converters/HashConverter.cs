using Microsoft.EntityFrameworkCore.Storage.ValueConversion;
using BerryAIGen.Domain.ValueObjects;
using System.Reflection;

namespace BerryAIGen.Infrastructure.Data.Converters;

/// <summary>
/// Value converter for Hash value object.
/// Uses reflection to bypass factory method with optional parameters.
/// </summary>
public class HashConverter : ValueConverter<Hash, string>
{
    private static readonly TypeInfo HashTypeInfo = typeof(Hash).GetTypeInfo();
    private static readonly ConstructorInfo HashConstructor = HashTypeInfo
        .DeclaredConstructors
        .First(c => c.GetParameters().Length == 2);

    /// <summary>
    /// Initializes a new instance of the HashConverter class.
    /// </summary>
    public HashConverter()
        : base(
            hash => hash.Value,
            value => CreateHash(value))
    {
    }

    private static Hash CreateHash(string value)
    {
        // Use private constructor to bypass factory validation
        // Validation already happened when entity was created
        return (Hash)HashConstructor.Invoke(new object[] { value, "SHA256" });
    }
}