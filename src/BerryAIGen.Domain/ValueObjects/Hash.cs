using BerryAIGen.Domain.Common;

namespace BerryAIGen.Domain.ValueObjects;

/// <summary>
/// Represents a cryptographic hash value for file integrity and identification.
/// </summary>
public sealed class Hash : ValueObject
{
    /// <summary>
    /// Gets the hash value as a hexadecimal string.
    /// </summary>
    public string Value { get; }

    /// <summary>
    /// Gets the algorithm used to generate the hash.
    /// </summary>
    public string Algorithm { get; }

    /// <summary>
    /// Initializes a new instance of the <see cref="Hash"/> class.
    /// </summary>
    /// <param name="value">The hash value.</param>
    /// <param name="algorithm">The hash algorithm (e.g., "SHA256", "MD5").</param>
    private Hash(string value, string algorithm)
    {
        Value = value ?? throw new ArgumentNullException(nameof(value));
        Algorithm = algorithm ?? throw new ArgumentNullException(nameof(algorithm));
    }

    /// <summary>
    /// Creates a Hash from a hexadecimal string.
    /// </summary>
    /// <param name="value">The hash value as hex string.</param>
    /// <param name="algorithm">The hash algorithm used.</param>
    /// <returns>A Hash instance.</returns>
    /// <exception cref="ArgumentException">Thrown when the hash value is invalid.</exception>
    public static Hash From(string value, string algorithm = "SHA256")
    {
        if (string.IsNullOrWhiteSpace(value))
            throw new ArgumentException("Hash value cannot be null or whitespace.", nameof(value));

        if (string.IsNullOrWhiteSpace(algorithm))
            throw new ArgumentException("Hash algorithm cannot be null or whitespace.", nameof(algorithm));

        // Validate that the value is a valid hexadecimal string
        if (!IsHexString(value))
            throw new ArgumentException("Hash value must be a valid hexadecimal string.", nameof(value));

        return new Hash(value.ToUpperInvariant(), algorithm.ToUpperInvariant());
    }

    /// <summary>
    /// Creates a SHA256 hash from a byte array.
    /// </summary>
    /// <param name="bytes">The bytes to hash.</param>
    /// <returns>A Hash instance.</returns>
    public static Hash FromBytes(byte[] bytes)
    {
        var hashBytes = System.Security.Cryptography.SHA256.HashData(bytes);
        var hexString = Convert.ToHexString(hashBytes);
        return new Hash(hexString, "SHA256");
    }

    /// <summary>
    /// Checks if a string is a valid hexadecimal string.
    /// </summary>
    private static bool IsHexString(string value)
    {
        return value.All(c => char.IsAsciiHexDigit(c));
    }

    /// <inheritdoc/>
    protected override IEnumerable<object?> GetEqualityComponents()
    {
        yield return Value;
        yield return Algorithm;
    }

    /// <summary>
    /// Returns the string representation of the hash.
    /// </summary>
    public override string ToString() => $"{Algorithm}:{Value}";

    /// <summary>
    /// Implicitly converts Hash to string (returns just the hash value).
    /// </summary>
    public static implicit operator string(Hash hash) => hash.Value;
}