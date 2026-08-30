using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_PlacedBuildAdded
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.PlacedBuildAdded); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.PlacedBuildAdded)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize FactoryEntityId
            s.Write(value.FactoryEntityId);
            //  Serialize EntityType
            s.Write(value.EntityType);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize BuildTime
            s.Write(value.BuildTime);
            //  Serialize LowPowerPenaltyBuildTime
            s.Write(value.LowPowerPenaltyBuildTime);
            //  Serialize BuildSpeed
            s.Write(value.BuildSpeed);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.PlacedBuildAdded)) as Rts.CnC.Messages.Client.PlacedBuildAdded;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize FactoryEntityId
            s.Read(out value.FactoryEntityId);
            //  Deserialize EntityType
            s.Read(out value.EntityType);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize BuildTime
            s.Read(out value.BuildTime);
            //  Deserialize LowPowerPenaltyBuildTime
            s.Read(out value.LowPowerPenaltyBuildTime);
            //  Deserialize BuildSpeed
            s.Read(out value.BuildSpeed);

            return value;
        }
        
    }
}
