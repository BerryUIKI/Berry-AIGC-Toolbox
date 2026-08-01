using Microsoft.EntityFrameworkCore.Storage.ValueConversion;
using BerryAIGen.Domain.ValueObjects;

namespace BerryAIGen.Infrastructure.Data.Converters;

/// <summary>
/// Value converter for Rating value object.
/// </summary>
public class RatingConverter : ValueConverter<Rating, int>
{
    /// <summary>
    /// Initializes a new instance of the RatingConverter class.
    /// </summary>
    public RatingConverter()
        : base(
            rating => rating.Value,
            value => Rating.From(value))
    {
    }
}