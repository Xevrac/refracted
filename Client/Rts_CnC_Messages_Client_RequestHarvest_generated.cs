using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestHarvest
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestHarvest); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestHarvest)obj;
            //  Serialize HarvesterPlayerId
            s.Write(value.HarvesterPlayerId);
            //  Serialize HarvesterId
            s.Write(value.HarvesterId);
            //  Serialize ResourcePlayerId
            s.Write(value.ResourcePlayerId);
            //  Serialize ResourceId
            s.Write(value.ResourceId);
            //  Serialize ResourceType
            s.Write(value.ResourceType);
            //  Serialize ModifierFlags
            s.WriteEnum(value.ModifierFlags);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestHarvest)) as Rts.CnC.Messages.Client.RequestHarvest;
            //  Deserialize HarvesterPlayerId
            s.Read(out value.HarvesterPlayerId);
            //  Deserialize HarvesterId
            s.Read(out value.HarvesterId);
            //  Deserialize ResourcePlayerId
            s.Read(out value.ResourcePlayerId);
            //  Deserialize ResourceId
            s.Read(out value.ResourceId);
            //  Deserialize ResourceType
            s.Read(out value.ResourceType);
            //  Deserialize ModifierFlags
            s.ReadEnum(out value.ModifierFlags);

            return value;
        }
        
    }
}
