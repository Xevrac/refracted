using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_StartHarvesting
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.StartHarvesting); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.StartHarvesting)obj;
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
            //  Serialize AmountToHarvest
            s.Write(value.AmountToHarvest);
            //  Serialize EngageTimeMS
            s.Write(value.EngageTimeMS);
            //  Serialize HarvestTimeMS
            s.Write(value.HarvestTimeMS);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.StartHarvesting)) as Rts.CnC.Messages.Client.StartHarvesting;
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
            //  Deserialize AmountToHarvest
            s.Read(out value.AmountToHarvest);
            //  Deserialize EngageTimeMS
            s.Read(out value.EngageTimeMS);
            //  Deserialize HarvestTimeMS
            s.Read(out value.HarvestTimeMS);

            return value;
        }
        
    }
}
