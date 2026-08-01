namespace BerryAIGen.Domain.ValueObjects;

/// <summary>
/// Represents a file path with validation and normalization.
/// </summary>
public sealed class FilePath : ValueObject
{
    /// <summary>
    /// Gets the normalized file path value.
    /// </summary>
    public string Value { get; }

    /// <summary>
    /// Initializes a new instance of the <see cref="FilePath"/> class.
    /// </summary>
    /// <param name="value">The file path value.</param>
    private FilePath(string value)
    {
        Value = value ?? throw new ArgumentNullException(nameof(value));
    }

    /// <summary>
    /// Creates a FilePath from a string value.
    /// </summary>
    /// <param name="value">The file path string.</param>
    /// <returns>A FilePath instance.</returns>
    /// <exception cref="ArgumentException">Thrown when the path is invalid.</exception>
    public static FilePath From(string value)
    {
        if (string.IsNullOrWhiteSpace(value))
            throw new ArgumentException("File path cannot be null or whitespace.", nameof(value));

        // Normalize the path
        var normalizedPath = System.IO.Path.GetFullPath(value);

        // Validate the path
        if (normalizedPath.Contains(".."))
            throw new ArgumentException("File path cannot contain relative path indicators.", nameof(value));

        return new FilePath(normalizedPath);
    }

    /// <summary>
    /// Gets the file name with extension.
    /// </summary>
    public string FileName => System.IO.Path.GetFileName(Value);

    /// <summary>
    /// Gets the file extension (lowercase, with dot).
    /// </summary>
    public string Extension => System.IO.Path.GetExtension(Value).ToLowerInvariant();

    /// <summary>
    /// Gets the directory path.
    /// </summary>
    public string DirectoryPath => System.IO.Path.GetDirectoryName(Value) ?? string.Empty;

    /// <inheritdoc/>
    protected override IEnumerable<object?> GetEqualityComponents()
    {
        yield return Value;
    }

    /// <summary>
    /// Returns the string representation of the file path.
    /// </summary>
    public override string ToString() => Value;

    /// <summary>
    /// Implicitly converts FilePath to string.
    /// </summary>
    public static implicit operator string(FilePath filePath) => filePath.Value;
}