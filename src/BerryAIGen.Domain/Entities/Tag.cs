using BerryAIGen.Domain.Common;
using BerryAIGen.Domain.Events;
using BerryAIGen.Domain.ValueObjects;

namespace BerryAIGen.Domain.Entities;

/// <summary>
/// Represents a tag for categorizing images.
/// </summary>
public sealed class Tag : Entity<TagId>
{
    /// <summary>
    /// Gets the name of the tag.
    /// </summary>
    public string Name { get; private set; }

    /// <summary>
    /// Gets the color of the tag (hex format, e.g., "#FF0000").
    /// </summary>
    public string? Color { get; private set; }

    /// <summary>
    /// Gets the category of the tag.
    /// </summary>
    public string? Category { get; private set; }

    /// <summary>
    /// Gets the creation timestamp.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Private constructor for EF Core.
    /// </summary>
    private Tag() { }

    /// <summary>
    /// Creates a new Tag.
    /// </summary>
    /// <param name="name">The tag name.</param>
    /// <param name="color">Optional color in hex format.</param>
    /// <param name="category">Optional category.</param>
    /// <returns>A new Tag instance.</returns>
    public static Tag Create(string name, string? color = null, string? category = null)
    {
        if (string.IsNullOrWhiteSpace(name))
            throw new ArgumentException("Tag name cannot be null or whitespace.", nameof(name));

        if (color != null && !IsValidHexColor(color))
            throw new ArgumentException("Color must be in hex format (e.g., #FF0000).", nameof(color));

        var tag = new Tag
        {
            Id = TagId.New(),
            Name = name.Trim(),
            Color = color?.ToUpperInvariant(),
            Category = category?.Trim(),
            CreatedAt = DateTime.UtcNow
        };

        return tag;
    }

    /// <summary>
    /// Renames the tag.
    /// </summary>
    /// <param name="newName">The new name.</param>
    public void Rename(string newName)
    {
        if (string.IsNullOrWhiteSpace(newName))
            throw new ArgumentException("Tag name cannot be null or whitespace.", nameof(newName));

        Name = newName.Trim();
    }

    /// <summary>
    /// Sets the tag color.
    /// </summary>
    /// <param name="color">The color in hex format.</param>
    public void SetColor(string? color)
    {
        if (color != null && !IsValidHexColor(color))
            throw new ArgumentException("Color must be in hex format (e.g., #FF0000).", nameof(color));

        Color = color?.ToUpperInvariant();
    }

    /// <summary>
    /// Sets the tag category.
    /// </summary>
    /// <param name="category">The category.</param>
    public void SetCategory(string? category)
    {
        Category = category?.Trim();
    }

    /// <summary>
    /// Validates hex color format.
    /// </summary>
    private static bool IsValidHexColor(string color)
    {
        return color.StartsWith("#") && color.Length == 7 && color.Skip(1).All(char.IsAsciiHexDigit);
    }
}